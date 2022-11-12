//! Launching mechanism for Helix.
//! This module crafts system calls to launch a new Minecraft instance.

// TODO: Make C API

use std::path::PathBuf;
// use std::slice::Join;
use std::vec::Vec;

#[cfg(target_os = "windows")]
static CLASSPATH_SEPARATOR: &str = ";";
#[cfg(not(target_os = "windows"))]
static CLASSPATH_SEPARATOR: &str = ":";

#[derive(Debug, Clone)]
pub struct ClassPath {
    pub path: PathBuf,
}

impl From<String> for ClassPath {
    fn from(path: String) -> Self {
        Self {
            path: PathBuf::from(path),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Launcher {
    /// Path to the Java executable
    // TODO: Change this later
    pub java: PathBuf,
    pub assets_dir: PathBuf,
    pub classpath: Vec<ClassPath>,
}

impl Launcher {
    pub fn new(java: PathBuf, assets_dir: PathBuf) -> Self {
        todo!()
    }

    pub fn launch(
        &self,
        version: &str,
        username: &str,
        session: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }

    pub fn generate_classpath(&self) -> String {
        let classpath = self
            .classpath
            .iter()
            .map(|cp| cp.path.to_str().unwrap())
            .collect::<Vec<&str>>()
            .join(CLASSPATH_SEPARATOR);
        classpath
    }
}
