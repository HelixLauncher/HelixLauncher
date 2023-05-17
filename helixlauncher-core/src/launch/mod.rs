//! Launching mechanism for Helix.
//! This module crafts system calls to launch a new Minecraft instance.

pub mod game;
pub mod instance;

// TODO: Make C API

use std::io;

use thiserror::Error;

#[cfg(target_os = "windows")]
const CLASSPATH_SEPARATOR: &str = ";";
#[cfg(not(target_os = "windows"))]
const CLASSPATH_SEPARATOR: &str = ":";

#[derive(Debug, Error)]
pub enum LaunchError {
    #[error("{0}")]
    IoError(#[from] io::Error),
}

fn generate_classpath(classpath: &[String]) -> String {
    classpath.join(CLASSPATH_SEPARATOR)
}
