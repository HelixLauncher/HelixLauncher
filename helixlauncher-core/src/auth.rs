pub mod account;
mod request_structs;

use reqwest::{Client, StatusCode};
use serde_json::json;
use std::time::Duration;
use thiserror::Error;

use account::Account;
use request_structs::*;

pub const DEFAULT_ACCOUNT_JSON: &str = "accounts.helix.json";

#[derive(Error, Debug)]
pub enum AuthenticationError {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),

    #[error("OAuth request was declined by the user.")]
    OAuthDeclined,

    #[error("OAuth request timed out, the user took too long.")]
    OAuthExpired,

    #[error("A request returned an unexpected result.")]
    Unexpected,
}

pub struct MinecraftAuthenticator {
    client_id: String,
    reqwest_client: Client,
}

impl MinecraftAuthenticator {
    pub fn new<I: Into<String>>(client_id: I) -> Self {
        Self {
            client_id: client_id.into(),
            reqwest_client: Client::new(),
        }
    }

    pub async fn initial_auth(
        &self,
        callback: fn(code: String, uri: String, message: String),
    ) -> Result<Account, AuthenticationError> {
        let code_response: CodeResponse = self
            .reqwest_client
            .get("https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode")
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("scope", "XboxLive.signin offline_access"),
            ])
            .send()
            .await?
            .json()
            .await?;

        callback(
            code_response.user_code,
            code_response.verification_uri,
            code_response.message,
        );

        loop {
            tokio::time::sleep(Duration::from_secs(code_response.interval as u64)).await;

            let grant_response = self
                .reqwest_client
                .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
                .form(&[
                    ("client_id", self.client_id.as_str()),
                    ("device_code", code_response.device_code.as_str()),
                    ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ])
                .send()
                .await?;

            return if grant_response.status() == StatusCode::OK {
                let poll_success: GrantSuccessResponse = grant_response.json().await?;

                self.authenticate(poll_success.access_token, poll_success.refresh_token)
                    .await
            } else {
                let poll_error: GrantFailureResponse = grant_response.json().await?;

                match poll_error.error.as_str() {
                    "authorization_pending" => continue,
                    "authorization_declined" => Err(AuthenticationError::OAuthDeclined),
                    "expired_token" => Err(AuthenticationError::OAuthExpired),
                    _ => Err(AuthenticationError::Unexpected),
                }
            };
        }
    }

    pub async fn refresh(&self, account: Account) -> Result<Account, AuthenticationError> {
        let grant_response: GrantSuccessResponse = self
            .reqwest_client
            .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("grant_type", "refresh_token"),
                ("scope", "XboxLive.signin offline_access"),
                ("refresh_token", account.refresh_token.as_str()),
            ])
            .send()
            .await?
            .json()
            .await?;

        self.authenticate(grant_response.access_token, grant_response.refresh_token)
            .await
    }

    async fn authenticate(
        &self,
        access_token: String,
        refresh_token: String,
    ) -> Result<Account, AuthenticationError> {
        let xbox_live_response: XboxLiveResponse = self
            .reqwest_client
            .post("https://user.auth.xboxlive.com/user/authenticate")
            .json(&json!({
                "Properties": {
                    "AuthMethod": "RPS",
                    "SiteName": "user.auth.xboxlive.com",
                    "RpsTicket": &format!("d={access_token}")
                },
                "RelyingParty": "http://auth.xboxlive.com",
                "TokenType": "JWT"
            }))
            .send()
            .await?
            .json()
            .await?;

        // Reuse the struct here, the response is laid out the same
        let xsts_response: XboxLiveResponse = self
            .reqwest_client
            .post("https://xsts.auth.xboxlive.com/xsts/authorize")
            .json(&json!({
                "Properties": {
                    "SandboxId": "RETAIL",
                    "UserTokens": [ xbox_live_response.token ]
                },
                "RelyingParty": "rp://api.minecraftservices.com/",
                "TokenType": "JWT"
            }))
            .send()
            .await?
            .json()
            .await?;

        let identity_token = format!(
            "XBL3.0 x={};{}",
            xsts_response.display_claims["xui"][0]["uhs"], xsts_response.token
        );
        let minecraft_response: MinecraftResponse = self
            .reqwest_client
            .post("https://api.minecraftservices.com/authentication/login_with_xbox")
            .json(&json!({ "identityToken": &identity_token }))
            .send()
            .await?
            .json()
            .await?;

        let profile_response: ProfileResponse = self
            .reqwest_client
            .get("https://api.minecraftservices.com/minecraft/profile")
            .header(
                "Authorization",
                format!("Bearer {}", minecraft_response.access_token),
            )
            .send()
            .await?
            .json()
            .await?;

        Ok(Account {
            uuid: profile_response.id,
            username: profile_response.name,
            refresh_token,
            token: minecraft_response.access_token,
            selected: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::auth::MinecraftAuthenticator;

    use super::{
        account::{add_account, get_accounts},
        DEFAULT_ACCOUNT_JSON,
    };

    #[tokio::test]
    #[ignore = "broken"]
    async fn test() {
        let authenticator = MinecraftAuthenticator::new("1d644380-5a23-4a84-89c3-5d29615fbac2");

        let account = authenticator
            .initial_auth(|_, _, message| println!("{}", message))
            .await
            .unwrap();

        println!("{}", serde_json::to_string(&account).unwrap());

        let account = authenticator.refresh(account).await.unwrap();

        println!("{}", serde_json::to_string(&account).unwrap());

        add_account(account, Path::new(DEFAULT_ACCOUNT_JSON)).unwrap();
    }

    #[test]
    fn test_account_storage() {
        for account in get_accounts(Path::new(DEFAULT_ACCOUNT_JSON)).unwrap() {
            println!("{}", serde_json::to_string(&account).unwrap());
        }
    }
}
