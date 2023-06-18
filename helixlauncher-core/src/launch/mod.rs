//! Launching mechanism for Helix.
//! This module crafts system calls to launch a new Minecraft instance.

pub mod asset;
pub mod instance;
pub mod prepared;

// TODO: Make C API

use std::{
    io,
    path::{self, Path},
};

use anyhow::Result;
use digest::Digest;
use helixlauncher_meta::component::{self, Hash};
use hex::ToHex;
use thiserror::Error;
use tokio::fs;

#[cfg(target_os = "windows")]
const CLASSPATH_SEPARATOR: &str = ";";
#[cfg(not(target_os = "windows"))]
const CLASSPATH_SEPARATOR: &str = ":";

#[derive(Debug, Error)]
pub enum LaunchError {
    #[error("Download of {url} failed: expected file with hash {expected_hash} and size {expected_size}, found file with hash {actual_hash} and size {actual_size}")]
    InvalidFile {
        url: String,
        expected_hash: component::Hash,
        expected_size: u32,
        actual_hash: String,
        actual_size: usize,
    },
    #[error("Invalid filename found: {name}")]
    InvalidFilename { name: String },
    #[error("Feature not supported by the instance: {name}")]
    UnsupportedFeature { name: String },
    #[error("{0}")]
    IoError(#[from] io::Error),
}

fn generate_classpath(classpath: &[String]) -> String {
    classpath.join(CLASSPATH_SEPARATOR)
}

async fn download_file(
    client: &reqwest::Client,
    path: &Path,
    url: &str,
    size: u32,
    hash: &component::Hash,
) -> Result<()> {
    if check_file(path, size, hash).await? {
        return Ok(());
    }

    fs::create_dir_all(path.parent().unwrap()).await?;

    println!("downloading: {}", url);

    let data = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    let (hash_matches, actual_hash) = check_hash(&data, hash);

    if data.len() != size as usize || !hash_matches {
        return Err(LaunchError::InvalidFile {
            url: url.to_string(),
            expected_hash: hash.clone(),
            expected_size: size,
            actual_hash,
            actual_size: data.len(),
        })?;
    }

    fs::write(path, data).await?;

    println!("download finished: {}", url);

    Ok(())
}

async fn check_file(path: &Path, size: u32, hash: &component::Hash) -> Result<bool, io::Error> {
    // This can be tricked by modifying or deleting the file after or while it is being processed
    // during launch, but let's not consider that an issue.

    // TODO: maybe not read in the entire file at once?
    if !path.try_exists()? {
        return Ok(false);
    }

    let file = match fs::read(path).await {
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(false),
        r => r,
    }?;
    Ok(file.len() == (size as usize) && check_hash(&file, hash).0)
}

fn check_hash(buf: &[u8], hash: &component::Hash) -> (bool, String) {
    let (expected, actual) = match hash {
        Hash::SHA1(hash) => (hash, sha1::Sha1::digest(buf).encode_hex::<String>()),
        Hash::SHA256(hash) => (hash, sha2::Sha256::digest(buf).encode_hex::<String>()),
    };
    (expected == &actual, actual)
}

fn copy_file(from: &Path, to: &Path) -> io::Result<()> {
    // TODO: investigate reflinks
    std::fs::copy(from, to)?;
    Ok(())
}

const ILLEGAL_FILENAMES: &[&str] = &[
    "aux", "com0", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8", "com9", "con",
    "lpt0", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9", "nul", "prn",
];

// This is not an exhaustive check on being a _valid_ path, but should be one on being a
// _dangerous_ path.
fn check_path(path: &str) -> bool {
    if path.is_empty() {
        return false;
    }
    for component in Path::new(path).components() {
        if let path::Component::Normal(component) = component {
            let name = component.to_str().unwrap();
            if name.is_empty()
                || name.contains(|c| matches!(c, '\0'..='\x1f' | '$' | ':' | '\\' | '/'))
            {
                return false;
            }
            let prefix = name.split_once('.').map_or(name, |t| t.0);
            if ILLEGAL_FILENAMES
                .binary_search(&prefix.to_ascii_lowercase().as_ref())
                .is_ok()
            {
                return false;
            }
        } else {
            return false;
        }
    }
    true
}
