use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Account {
    pub uuid: String,
    pub username: String,
    pub refresh_token: String,
    pub token: String,
}
