use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufReader, path::Path};
use thiserror::Error;

#[derice(Error, Debug)]
pub enum AccountManagerError {
    #[error(Transparent)]
    ParseError(#[from] serde_json::Error),

    #[error(Transparent)]
    FileError(#[from] std::io::Error),
}

#[derive(Serialize, Deserialize)]
pub struct Account {
    pub uuid: String,
    pub username: String,
    pub refresh_token: String,
    pub token: String,
}

pub fn get_accounts(account_json: &Path) -> Result<Vec<Account>, AccountManagerError> {
    serde_json::from_reader(BufReader::new(File::open(account_json)?))?
}

pub fn add_account(account: Account, account_json: &Path) -> Result<()> {
    let mut accounts: Vec<Account> =
        serde_json::from_reader(BufReader::new(File::open(account_json)?))?;

    accounts.push(account);

    let writer = File::create(account_json)?;
    serde_json::to_writer_pretty(writer, &accounts)?;

    Ok(())
}
