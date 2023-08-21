use std::io;

use reqwest::Client;
use thiserror::Error;

use crate::config::Config;

pub struct HelixLauncherMeta<'a> {
    client: Client,
    config: &'a Config,
}

impl<'a> HelixLauncherMeta<'a> {
    pub fn new(config: &'a Config) -> HelixLauncherMeta<'a> {
        HelixLauncherMeta {
            client: Client::new(),
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
                    "{}{}/{}.json",
                    self.config.get_meta_url(),
                    component_id,
                    component_version
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

        path.push(format!("{}.json", component_version));

        let component_data = match component_data_result {
            Err(e) => match tokio::fs::read(path).await {
                Err(_) => Err(e)?,
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
        Ok(self
            .client
            .get(format!(
                "{}{}.json",
                self.config.get_meta_url(),
                component_id
            ))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
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
}
