mod request_structs;
mod account;

use std::thread;
use std::time::Duration;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

use crate::authentication::account::Account;
use crate::authentication::request_structs::{
	CodeResponse,
	MinecraftResponse,
	PollErrorResponse,
	PollSuccessResponse,
	ProfileResponse,
	XboxLiveResponse
};

#[derive(Error, Debug)]
pub enum AuthenticationError {
	#[error(transparent)]
	ReqwestError(#[from] reqwest::Error),

	#[error("OAuth request was declined by the user.")]
	OAuthDeclined,

	#[error("OAuth request timed out, the user took too long.")]
	OAuthExpired,

	#[error("A request returned an unexpected result.")]
	Unexpected
}

pub struct MinecraftAuthenticator {
	client_id: String,
	reqwest_client: Client
}

impl MinecraftAuthenticator {
	pub fn new(client_id: &str) -> Self {
		MinecraftAuthenticator { client_id: client_id.to_string(), reqwest_client: Client::new() }
	}

	pub async fn initial_auth(self, callback: fn(code: String, uri: String, message: String)) -> Result<Account, AuthenticationError> {
		let code_response: CodeResponse = self.reqwest_client
			.get("https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode")
			.form(&vec![
				("client_id", self.client_id.as_str()),
				("scope", "XboxLive.signin offline_access")
			])
			.send().await?.json().await?;

		callback(code_response.user_code, code_response.verification_uri, code_response.message);

		loop {
			thread::sleep(Duration::from_secs(code_response.interval as u64));

			let poll_response = self.reqwest_client
				.post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
				.form(&vec![
					("client_id", self.client_id.as_str()),
					("device_code", code_response.device_code.as_str()),
					("grant_type", &"urn:ietf:params:oauth:grant-type:device_code")
				])
				.send().await?;

			match poll_response.status().as_u16() {
				200 => {
					let poll_success: PollSuccessResponse = poll_response.json().await?;

					let xbox_live_response: XboxLiveResponse = self.reqwest_client
						.post("https://user.auth.xboxlive.com/user/authenticate")
						.json(&json!({
							"Properties": {
								"AuthMethod": "RPS",
								"SiteName": "user.auth.xboxlive.com",
								"RpsTicket": &format!("d={}", &poll_success.access_token)
							},
							"RelyingParty": "http://auth.xboxlive.com",
							"TokenType": "JWT"
						}))
						.send().await?.json().await?;

					// Reuse the struct here, the response is laid out the same
					let xsts_response: XboxLiveResponse = self.reqwest_client
						.post("https://xsts.auth.xboxlive.com/xsts/authorize")
						.json(&json!({
							"Properties": {
								"SandboxId": "RETAIL",
								"UserTokens": [ xbox_live_response.token ]
							},
							"RelyingParty": "rp://api.minecraftservices.com/",
							"TokenType": "JWT"
						}))
						.send().await?.json().await?;

					let minecraft_response: MinecraftResponse = self.reqwest_client
						.post("https://api.minecraftservices.com/authentication/login_with_xbox")
						.json(&json!({
							"identityToken": &format!(
								"XBL3.0 x={};{}",
								xsts_response.display_claims["xui"][0]["uhs"],
								xsts_response.token
							)
						}))
						.send().await?.json().await?;

					let profile_response: ProfileResponse = self.reqwest_client
						.get("https://api.minecraftservices.com/minecraft/profile")
						.header("Authorization", format!("Bearer {}", minecraft_response.access_token))
						.send().await?.json().await?;

					return Ok(Account {
						uuid: profile_response.id,
						username: profile_response.name,

						refresh_token: poll_success.refresh_token,

						token: minecraft_response.access_token
					})
				}
				_ => {
					let poll_error: PollErrorResponse = poll_response.json().await?;

					match poll_error.error.as_str() {
						"authorization_pending" => continue,
						"authorization_declined" => return Err(AuthenticationError::OAuthDeclined),
						"expired_token" => return Err(AuthenticationError::OAuthExpired),
						_ => return Err(AuthenticationError::Unexpected)
					}
				}
			}
		}
	}
}