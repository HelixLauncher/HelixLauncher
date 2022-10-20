use std::collections::HashMap;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct DeviceCodeResponse {
	// This information is only needed by the authenticator, the application using the API does not
	// need this information.
	pub(crate) device_code: String,
	pub(crate) interval: i32,

	pub user_code: String,
	pub verification_uri: String,
	pub message: String
}

#[derive(Deserialize)]
pub(crate) struct PollSuccessResponse {
	pub(crate) access_token: String
}

#[derive(Deserialize)]
pub(crate) struct PollErrorResponse {
	pub(crate) error: String
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct XboxLiveResponse {
	pub(crate) token: String,
	pub(crate) display_claims: HashMap<String, Vec<HashMap<String, String>>>
}

#[derive(Deserialize)]
pub(crate) struct MinecraftResponse {
	pub(crate) access_token: String,
}

#[derive(Deserialize)]
pub(crate) struct ProfileResponse {
	pub(crate) id: String,
	pub(crate) name: String
}