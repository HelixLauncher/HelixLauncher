//! Initial config support for libhelix
//!
//! TODO:
//! - Allow for users to provide their own path
//! - make sure get_base_path doesn't panic
//! - add fields the rest of the fields into Config

use serde::{Deserialize, Serialize};
use std::env;
use std::io::BufReader;
use std::path::PathBuf;
use std::{
    fs::{self, File},
    io,
};

pub const CONFIG_NAME: &str = "config.helix.json";

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    // the base path shouldn't be saved in the file, as the data dir may move
    #[serde(skip)]
    base_path: PathBuf,

    // settings... feel free to add fields as required
    #[serde(default = "instances_default")]
    instances_dir: PathBuf,
    #[serde(default = "libraries_default")]
    libraries_dir: PathBuf,
    #[serde(default = "assets_default")]
    assets_dir: PathBuf,
}

fn instances_default() -> PathBuf {
    PathBuf::from("instances")
}

fn libraries_default() -> PathBuf {
    PathBuf::from("libraries")
}

fn assets_default() -> PathBuf {
    PathBuf::from("assets")
}

impl Config {
    /// `appdir` is the rDNS name of your application, also used as the macOS bundle id or the
    /// `.desktop` file name on Linux. It will be used in the location of the data folder on macOS.
    /// `name` is the name of your application and will be used in the location of the data folder
    /// on Linux and Windows.
    pub fn new(appid: &str, name: &str) -> Result<Self, Error> {
        // TODO allow the user to provide their own path
        let mut path = get_base_path();
        path.push(if cfg!(any(target_os = "macos", target_os = "ios")) {
            appid
        } else {
            name
        });

        Self::new_with_data_dir(appid, name, path)
    }

    pub fn new_with_data_dir(_appid: &str, _name: &str, path: PathBuf) -> Result<Self, Error> {
        let config = if !path.join(CONFIG_NAME).exists() {
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
        let filepath = self.base_path.join(CONFIG_NAME);

        let mut file = File::create(filepath)?;
        serde_json::to_writer_pretty(&mut file, self)?;

        Ok(())
    }

    pub fn read_config<P: Into<PathBuf>>(base_path: P) -> io::Result<Self> {
        let base_path: PathBuf = base_path.into();

        let file = File::open(base_path.join(CONFIG_NAME))?;
        let mut read: Self = serde_json::from_reader(BufReader::new(file))?;
        read.base_path = base_path;

        Ok(read)
    }

    fn default_config(base_path: PathBuf) -> Self {
        Self {
            base_path,
            instances_dir: PathBuf::from("instances"),
            libraries_dir: PathBuf::from("libraries"),
            assets_dir: PathBuf::from("assets"),
        }
    }

    pub fn get_base_path(&self) -> &PathBuf {
        &self.base_path
    }

    pub fn get_instances_path(&self) -> PathBuf {
        self.base_path.join(&self.instances_dir)
    }

    pub fn get_libraries_path(&self) -> PathBuf {
        self.base_path.join(&self.libraries_dir)
    }

    pub fn get_assets_path(&self) -> PathBuf {
        self.base_path.join(&self.assets_dir)
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
    dirs::data_dir()
        .or_else(|| env::current_dir().ok())
        .unwrap()
}

#[cfg(test)]
mod tests {
    use anyhow::{Result, Ok};

    use crate::config::Config;
    
    #[tokio::test]
    async fn create_config_in_non_existing_dir() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let _ =  Config::new_with_data_dir("","",dir.path().join("abc"))?;
        Ok(())
    }

    #[tokio::test]
    async fn create_config_in_existing_dir() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let _ =  Config::new_with_data_dir("","",dir.path().to_path_buf())?;
        Ok(())
    }
}