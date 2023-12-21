use std::io;

use thiserror::Error;

use crate::config::{self, Config};

pub struct MetaClient<'a> {
    client: reqwest::Client,
    config: &'a Config,
}

impl<'a> MetaClient<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self {
            client: reqwest::Client::new(),
            config,
        }
    }

    pub async fn get_component_meta(
        &self,
        component_id: &str,
        component_version: &str,
    ) -> Result<helixlauncher_meta::component::Component, ComponentMetaRetrievalError> {
        let config_meta = self.config.get_meta_url();
        let meta_default = config::meta_url_default();
        let url = if let Some(configured_url) = config_meta {
            configured_url
        } else {
            meta_default.as_str()
        };
        // TODO: better caching
        let component_data_result = async {
            self.client
                .get(format!("{url}{component_id}/{component_version}.json"))
                .send()
                .await?
                .error_for_status()?
                .bytes()
                .await
        }
        .await;

        let mut path = self.config.get_base_path().join("meta");
        path.push(component_id);

        tokio::fs::create_dir_all(&path).await?;

        path.push(format!("{component_version}.json"));

        let component_data = match component_data_result {
            Err(e) => match tokio::fs::read(path).await {
                Err(_) => {
                    if e.status() == Some(reqwest::StatusCode::NOT_FOUND) {
                        return Err(ComponentMetaRetrievalError::VersionNotFound {
                            id: component_id.to_string(),
                            version: component_version.to_string(),
                        });
                    } else {
                        return Err(e.into());
                    }
                }
                Ok(r) => r,
            },
            Ok(r) => {
                tokio::fs::write(path, &r).await?;
                r.into()
            }
        };

        Ok(serde_json::from_slice(&component_data)?)
    }

    pub async fn get_component_index(
        &self,
        component_id: &str,
    ) -> Result<helixlauncher_meta::index::Index, ComponentMetaRetrievalError> {
        let config_meta = self.config.get_meta_url();
        let meta_default = config::meta_url_default();
        let url = if let Some(configured_url) = config_meta {
            configured_url
        } else {
            meta_default.as_str()
        };
        let response = self
            .client
            .get(format!("{url}{component_id}/index.json"))
            .send()
            .await?
            .error_for_status();
        let response = match response {
            Ok(r) => r,
            Err(e) => {
                if e.status() == Some(reqwest::StatusCode::NOT_FOUND) {
                    return Err(ComponentMetaRetrievalError::IndexNotFound {
                        id: component_id.to_string(),
                    });
                } else {
                    return Err(e.into());
                }
            }
        };
        Ok(response.json().await?)
    }

    pub async fn component_version_exists(
        &self,
        component_id: &str,
        component_version: &str,
    ) -> Result<bool, ComponentMetaRetrievalError> {
        Ok(self
            .get_component_index(component_id)
            .await?
            .iter()
            .any(|item| item.version == component_version))
    }
}

#[derive(Error, Debug)]
pub enum ComponentMetaRetrievalError {
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error(transparent)]
    ParseError(#[from] serde_json::Error),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error("Version {version} of component {id} not found")]
    VersionNotFound { id: String, version: String },
    #[error("Component {id} not found")]
    IndexNotFound { id: String },
}

#[cfg(test)]
mod tests {

    use crate::config::Config;

    use super::*;

    const VALID_TEST_SET: [(&str, [&str; 3]); 1] = [
        // TODO: adjust to meta changes
        ("net.minecraft", ["1.16.5", "23w18a", "3D Shareware v1.34"]),
        // ("org.quiltmc.quilt-loader", ["0.20.0-beta.2", "0.18.9", "0.18.1-beta.10"]),
        // ("net.fabricmc.fabric-loader", ["", "", ""]),
        // ("net.minecraftforge.forge", ["", "", ""]),
    ];

    #[tokio::test]
    async fn get_component_meta() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let config = Config::new_with_data_dir(
            "dev.helixlauncher.HelixLauncher",
            "HelixLauncher",
            dir.path().join("abc"),
        )?;

        for (component_id, versions) in VALID_TEST_SET {
            for component_version in versions {
                MetaClient::new(&config)
                    .get_component_meta(component_id, component_version)
                    .await?;
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn get_component_meta_same_client() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let config = Config::new_with_data_dir(
            "dev.helixlauncher.HelixLauncher",
            "HelixLauncher",
            dir.path().join("abc"),
        )?;
        let client = MetaClient::new(&config);
        for (component_id, versions) in VALID_TEST_SET {
            for component_version in versions {
                client
                    .get_component_meta(component_id, component_version)
                    .await?;
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn get_component_index() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let config = Config::new_with_data_dir(
            "dev.helixlauncher.HelixLauncher",
            "HelixLauncher",
            dir.path().join("abc"),
        )?;

        for (component_id, _) in VALID_TEST_SET {
            assert!(
                MetaClient::new(&config)
                    .get_component_index(component_id)
                    .await?
                    .len()
                    > 5
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn get_component_index_same_client() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let config = Config::new_with_data_dir(
            "dev.helixlauncher.HelixLauncher",
            "HelixLauncher",
            dir.path().join("abc"),
        )?;
        let client = MetaClient::new(&config);
        for (component_id, _) in VALID_TEST_SET {
            assert!(client.get_component_index(component_id).await?.len() > 5);
        }

        Ok(())
    }

    #[tokio::test]
    async fn component_version_exists() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let config = Config::new_with_data_dir(
            "dev.helixlauncher.HelixLauncher",
            "HelixLauncher",
            dir.path().join("abc"),
        )?;

        for (component_id, versions) in VALID_TEST_SET {
            for component_version in versions {
                assert!(
                    MetaClient::new(&config)
                        .component_version_exists(component_id, component_version)
                        .await?
                )
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn component_version_exists_same_client() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let config = Config::new_with_data_dir(
            "dev.helixlauncher.HelixLauncher",
            "HelixLauncher",
            dir.path().join("abc"),
        )?;

        let client = MetaClient::new(&config);
        for (component_id, versions) in VALID_TEST_SET {
            for component_version in versions {
                assert!(
                    client
                        .component_version_exists(component_id, component_version)
                        .await?
                )
            }
        }

        Ok(())
    }

    const INVALID_TEST_SET: [(&str, [&str; 3]); 2] = [
        // first should have invalid index, rest only invalid versions
        ("com.example", ["1.0.0", "1.0.1", "1.0.2"]),
        ("net.minecraft", ["4.0.0", "aklsnbafoibesgobo", "100+5=3"]),
    ];

    #[tokio::test]
    async fn get_component_meta_invalid_version() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let config = Config::new_with_data_dir(
            "dev.helixlauncher.HelixLauncher",
            "HelixLauncher",
            dir.path().join("abc"),
        )?;

        for (component_id, versions) in INVALID_TEST_SET {
            for component_version in versions {
                assert!(matches!(
                    MetaClient::new(&config)
                        .get_component_meta(component_id, component_version)
                        .await,
                    Err(ComponentMetaRetrievalError::VersionNotFound {
                        id,
                        version
                    }) if id == component_id && version == component_version
                ));
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn get_component_index_invalid_id() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let config = Config::new_with_data_dir(
            "dev.helixlauncher.HelixLauncher",
            "HelixLauncher",
            dir.path().join("abc"),
        )?;

        assert!(matches!(
            MetaClient::new(&config)
                .get_component_index(INVALID_TEST_SET[0].0)
                .await,
            Err(
                ComponentMetaRetrievalError::IndexNotFound{
                id
            }) if id == INVALID_TEST_SET[0].0
        ));
        Ok(())
    }

    #[tokio::test]
    async fn get_component_meta_invalid_formatting() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let config = Config::new_with_data_dir(
            "dev.helixlauncher.HelixLauncher",
            "HelixLauncher",
            dir.path().join("abc"),
        )?;

        assert!(matches!(
            MetaClient::new(&config)
                .get_component_meta("net.minecraft", "index")
                .await,
            Err(ComponentMetaRetrievalError::ParseError(_))
        ));

        Ok(())
    }
}
