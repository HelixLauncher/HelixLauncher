use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{self, BufReader},
    path::Path,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AccountManagerError {
    #[error(transparent)]
    ParseError(#[from] serde_json::Error),

    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Account {
    pub uuid: String,
    pub username: String,
    pub refresh_token: String,
    pub token: String,
    pub selected: bool,
}

pub fn get_accounts(account_json: &Path) -> Result<Vec<Account>, AccountManagerError> {
    // try to open file, return empty if file doesnt exist
    Ok(serde_json::from_reader(BufReader::new(
        match File::open(account_json) {
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(vec![]),
            result => result,
        }?,
    ))?)
}

pub fn add_account(mut account: Account, account_json: &Path) -> Result<(), AccountManagerError> {
    let mut accounts: Vec<Account> = get_accounts(account_json)?;
    if accounts.len() == 0 {
        account.selected = true
    }
    accounts.push(account);

    let writer = File::create(account_json)?;
    serde_json::to_writer_pretty(writer, &accounts)?;

    Ok(())
}
