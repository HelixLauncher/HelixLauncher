//! Launching mechanism for Helix.
//! This module crafts system calls to launch a new Minecraft instance.

// TODO: Make C API

use std::{io, process::Stdio};
use tokio::process::{Child, Command};

use crate::game::PreparedLaunch;
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

// TODO: add better API for log output
pub async fn launch(
    prepared_launch: &PreparedLaunch,
    inherit_out: bool,
) -> Result<Child, LaunchError> {
    if !inherit_out {
        todo!();
    }
    let classpath = generate_classpath(&prepared_launch.classpath);
    // TODO: hook up javalaunch
    Ok(Command::new(&prepared_launch.java_path)
        .args(&prepared_launch.jvm_args)
        .arg("-classpath")
        .arg(classpath)
        .arg(&prepared_launch.main_class)
        .args(&prepared_launch.args)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?)
}

fn generate_classpath(classpath: &[String]) -> String {
    classpath.join(CLASSPATH_SEPARATOR)
}
