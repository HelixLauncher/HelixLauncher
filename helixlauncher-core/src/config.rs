//! Initial config support for libhelix
//!
//! TODO:
//! - Allow for users to provide their own path
//! - make sure get_base_path doesn't panic
//! - add fields the rest of the fields into Config

use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use std::{
    fs::{self, File},
    io,
};

pub const CONFIG_NAME: &str = "config.helix.json";

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    // the path doesn't need to be saved in the file,
    // but it is useful to keep around imo
    #[serde(skip)]
    path: PathBuf, // profiles/instances
                   // settings... feel free to add fields as required
}

impl Config {
    pub fn new(name: &str) -> Result<Self, Error> {
        // TODO allow the user to provide their own path
        let mut path = get_base_path();
        path.push(name);

        let config = if !path.exists() {
            // Could return an error if the dir already exists,
            // but we don't care about that
            if let Err(e) = fs::create_dir_all(&path) {
                if e.kind() == io::ErrorKind::PermissionDenied {
                    return Err(Error::PermissionDenied(e));
                }
            }

            let conf = Self::default_config(path);
            conf.save_config()?;
            conf
        } else {
            Self::read_config(path)?
        };

        Ok(config)
    }

    pub fn save_config(&self) -> Result<(), Error> {
        let filepath = self.path.join(CONFIG_NAME);

        let mut file = File::options().write(true).create(true).open(filepath)?;
        serde_json::to_writer_pretty(&mut file, self)?;

        Ok(())
    }

    pub fn read_config<P: Into<PathBuf>>(path: P) -> io::Result<Self> {
        let path: PathBuf = path.into();

        let file = File::open(path.join(CONFIG_NAME))?;
        let mut read: Self = serde_json::from_reader(file)?;
        read.path = path;

        Ok(read)
    }

    fn default_config(path: PathBuf) -> Self {
        Self { path }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Could not find config file")]
    ConfigNotFound,
    #[error("Permission denied: {0}")]
    PermissionDenied(io::Error),
    #[error("An IO error occurred {0}")]
    IoError(io::Error),
    #[error("Serialization failed: {0}")]
    SerializeFailed(serde_json::Error),
    #[error("Deserialization failed: {0}")]
    DeserializeFailed(serde_json::Error),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        use io::ErrorKind;

        match e.kind() {
            ErrorKind::NotFound => Self::ConfigNotFound,
            ErrorKind::PermissionDenied => Self::PermissionDenied(e),
            _ => Self::IoError(e),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        use serde_json::error::Category;

        match e.classify() {
            Category::Syntax | Category::Data | Category::Eof => Self::DeserializeFailed(e),
            Category::Io => Self::IoError(e.into()),
        }
    }
}

fn get_base_path() -> PathBuf {
    // We should be using the dirs crate to get the base path.
    // This works on almost all platforms, especially windows and macOS.

    // See here: https://docs.rs/dirs/latest/dirs/fn.data_dir.html

    dirs::data_dir()
        .or_else(|| env::current_dir().ok())
        .unwrap()
    /*
    // This logic isn't perfect and could crash in its current state
    if cfg!(windows) {
        env::var("APPDATA").unwrap().into()
    } else {
        let home = env::var("HOME");

        match home {
            Ok(ok) => {
                let mut path: PathBuf = ok.into();
                path.push(".local");
                path.push("share");
                path
            }
            Err(_) => env::current_dir().unwrap(),
        }
    } */
}
