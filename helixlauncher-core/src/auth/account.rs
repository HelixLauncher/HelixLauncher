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
    pub default: Option<String>,
    #[serde(skip)]
    path: PathBuf,
}

impl AccountConfig {
    pub fn new(account_json: PathBuf) -> Result<AccountConfig, AccountManagerError> {
        let mut account_config: AccountConfig =
            serde_json::from_reader(BufReader::new(match File::open(&account_json) {
                Err(err) if err.kind() == io::ErrorKind::NotFound => {
                    return Ok(AccountConfig {
                        accounts: vec![],
                        path: account_json,
                        default: None,
                    })
                }
                result => result,
            }?))?;
        account_config.path = account_json;
        Ok(account_config)
    }

    pub fn save(&self) -> Result<(), AccountManagerError> {
        let writer = File::create(&self.path)?;
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }

    /// returns the account specified in the [default](`AccountConfig::default`) field, or the first, if only a single account exists and no default is specified
    pub fn selected(&self) -> Option<&Account> {
        match &self.default {
            Some(uuid) => self.accounts.iter().find(|it| it.uuid == *uuid),
            None => {
                if self.accounts.len() == 1 {
                    Some(&self.accounts[0])
                } else {
                    None
                }
            }
        }
    }
}
