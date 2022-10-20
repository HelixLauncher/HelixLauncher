use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MinecraftAuthenticatorError {
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

	pub fn authenticate(self, display_code: fn(DeviceCodeResponse)) -> Result<Account, MinecraftAuthenticatorError> {
		let device_code_response: DeviceCodeResponse = self.reqwest_client
			.get("https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode")
			.form(&vec![
				("client_id", self.client_id.as_str()),
				("scope", "XboxLive.signin")
			])
			.send()?.json()?;

		display_code(device_code_response.clone());

		loop {
			thread::sleep(Duration::from_secs(device_code_response.interval as u64));

			let poll_response = self.reqwest_client
				.post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
				.form(&vec![
					("client_id", self.client_id.as_str()),
					("device_code", device_code_response.device_code.as_str()),
					("grant_type", &"urn:ietf:params:oauth:grant-type:device_code")
				])
				.send()?;

			match poll_response.status().as_u16() {
				200 => {
					let poll_success: PollSuccessResponse = poll_response.json()?;

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
						.send()?.json()?;

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
						.send()?.json()?;

					let minecraft_response: MinecraftResponse = self.reqwest_client
						.post("https://api.minecraftservices.com/authentication/login_with_xbox")
						.json(&json!({
							"identityToken": &format!(
								"XBL3.0 x={};{}",
								xsts_response.display_claims["xui"][0]["uhs"],
								xsts_response.token
							)
						}))
						.send()?.json()?;

					let profile_response: ProfileResponse = self.reqwest_client
						.get("https://api.minecraftservices.com/minecraft/profile")
						.header("Authorization", format!("Bearer {}", minecraft_response.access_token))
						.send()?.json()?;

					return Ok(Account {
						uuid: profile_response.id,
						username: profile_response.name,
						minecraft_token: minecraft_response.access_token
					})
				}
				_ => {
					let poll_error: PollErrorResponse = poll_response.json()?;

					match poll_error.error.as_str() {
						"authorization_pending" => continue,
						"authorization_declined" => return Err(MinecraftAuthenticatorError::OAuthDeclined),
						"expired_token" => return Err(MinecraftAuthenticatorError::OAuthExpired),
						_ => return Err(MinecraftAuthenticatorError::Unexpected)
					}
				}
			}
		}
	}
}

#[derive(Deserialize, Clone)]
pub struct DeviceCodeResponse {
	// This information is only needed by the authenticator, the application using the API does not
	// need this information.
	device_code: String,
	interval: i32,
	expires_in: i32,

	pub user_code: String,
	pub verification_uri: String,
	pub message: String
}

#[derive(Deserialize)]
pub(crate) struct PollSuccessResponse {
	token_type: String,
	scope: String,
	expires_in: i32,
	ext_expires_in: i32,
	access_token: String,
	// refresh_token: String,
	// id_token: String
}

#[derive(Deserialize)]
pub(crate) struct PollErrorResponse {
	error: String
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct XboxLiveResponse {
	issue_instant: String,
	not_after: String,
	token: String,
	display_claims: HashMap<String, Vec<HashMap<String, String>>>
}

#[derive(Deserialize)]
pub(crate) struct MinecraftResponse {
	username: String,
	// roles: Vec<?>,
	// metadata: HashMap<?, ?>,
	access_token: String,
	expires_in: i32,
	token_type: String
}

#[derive(Deserialize)]
pub(crate) struct ProfileResponse {
	id: String,
	name: String,
	skins: Vec<Skin>,
	// capes: Vec<?>
}

#[derive(Deserialize)]
pub(crate) struct Skin {
	id: String,
	state: String,
	url: String,
	variant: String
}

#[derive(Serialize, Deserialize)]
pub struct Account {
	uuid: String,
	username: String,
	minecraft_token: String
}