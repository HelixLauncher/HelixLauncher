use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufReader, path::Path};

#[derive(Serialize, Deserialize)]
pub struct Account {
    pub uuid: String,
    pub username: String,
    pub refresh_token: String,
    pub token: String,
}

pub fn get_accounts(account_json: &Path) -> Vec<Account> {
    serde_json::from_reader(BufReader::new(File::open(account_json).unwrap())).unwrap()
}

pub fn add_account(account: Account, account_json: &Path) {
    let mut accounts: Vec<Account> =
        serde_json::from_reader(BufReader::new(File::open(account_json).unwrap())).unwrap();

    accounts.push(account);

    let writer = File::create(account_json).unwrap();
    serde_json::to_writer_pretty(writer, &accounts).unwrap();
}
