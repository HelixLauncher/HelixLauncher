use cfg_if::cfg_if;
use std::{
    collections::HashSet,
    fs,
    io::{Error as IOError, ErrorKind as IOErrorKind},
    path::{Path, PathBuf},
};
use thiserror::Error;

pub struct JavaInstallInfo {
    path: String,
    major_version: u32,
    version: String,
    architecture: helixlauncher_meta::component::Arch,
}

#[derive(Error, Debug)]
pub enum SearchError {
    #[error("{0}")]
    IO(#[from] IOError),
}

#[cfg(windows)]
const JAVA_EXECUTABLE: &str = "bin\\javaw.exe";
#[cfg(unix)]
const JAVA_EXECUTABLE: &str = "bin/java";

#[cfg(windows)]
fn search_registry(result: &mut HashSet<PathBuf>) {}

fn search_java_dir<P: AsRef<Path>>(
    result: &mut HashSet<PathBuf>,
    dir: P,
) -> Result<(), SearchError> {
    for dir in match fs::read_dir(dir) {
        Err(err) if err.kind() == IOErrorKind::NotFound => return Ok(()),
        r => r,
    }? {
        let dir = match dir {
            Err(err) if err.kind() == IOErrorKind::NotFound => continue,
            r => r,
        }?;
        let mut path = dir.path();
        if !path.is_dir() {
            continue;
        }
        path.push(JAVA_EXECUTABLE);
        if path.exists() {
            result.insert(path);
        }
    }
    Ok(())
}

fn search_java_dirs(result: &mut HashSet<PathBuf>) -> Result<(), SearchError> {
    cfg_if! {
        if #[cfg(windows)] {
        } else if #[cfg(unix)] {
            #[cfg(target_os = "osx")] {
            }
            search_java_dir(result, "/usr/lib/jvm")?;
        } else {
            compile_error!("Unknown platform");
        }
    }
    Ok(())
}

pub fn search_java() -> Result<HashSet<PathBuf>, SearchError> {
    let mut result = HashSet::new();
    search_java_dirs(&mut result)?;
    Ok(result)
}

pub async fn java_info(path: &Path) {}
