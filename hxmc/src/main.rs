//! Helix Launcher CLI
//! This is an example implementation of the Helix Launcher CLI.

use anyhow::{Ok, Result};
use clap::{Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};

use log::info;
#[derive(Parser, Debug)]
struct HelixLauncher {
    #[command(subcommand)]
    subcommand: Command,

    #[command(flatten)]
    verbosity: Verbosity<InfoLevel>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Launches a new Helix instance
    Start,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = HelixLauncher::parse();
    pretty_env_logger::formatted_builder()
        .filter_level(cli.verbosity.log_level_filter())
        .init();
    info!("Hello world!");
    Ok(())
}
