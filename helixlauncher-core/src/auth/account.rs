use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{self, BufReader},
    path::PathBuf,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AccountManagerError {
    #[error(transparent)]
    ParseError(#[from] serde_json::Error),

    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Account {
    pub uuid: String,
    pub username: String,
    pub refresh_token: String,
    pub token: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccountConfig {
    pub accounts: Vec<Account>,
    /// The UUID of the selected account
    pub selected: String,
    #[serde(skip)]
    path: PathBuf,
}

impl AccountConfig {
    pub fn new(account_json: PathBuf) -> Result<AccountConfig, AccountManagerError> {
        Ok(serde_json::from_reader(BufReader::new(
            match File::open(&account_json) {
                Err(err) if err.kind() == io::ErrorKind::NotFound => {
                    return Ok(AccountConfig {
                        accounts: vec![],
                        path: account_json,
                        selected: String::new(),
                    })
                }
                result => result,
            }?,
        ))?)
    }

    pub fn save(&self) -> Result<(), AccountManagerError> {
        let writer = File::create(&self.path)?;
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }
}
