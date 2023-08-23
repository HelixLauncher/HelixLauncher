use std::io;

use thiserror::Error;

use crate::config::Config;

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
        // TODO: better caching
        let component_data_result = async {
            self.client
                .get(format!(
                    "{}{component_id}/{component_version}.json",
                    self.config.get_meta_url()
                ))
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
        let response = self
            .client
            .get(format!("{}{component_id}.json", self.config.get_meta_url()))
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
        component_id: String,
        component_version: String,
    ) -> Result<bool, ComponentMetaRetrievalError> {
        Ok(self
            .get_component_index(&component_id)
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
