//! Helix Launcher CLI
//! This is an example implementation of the Helix Launcher CLI.

use std::fs;
use std::io;

use anyhow::{Ok, Result};
use clap::{Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use helixlauncher_core::instance::Instance;
use helixlauncher_core::instance::InstanceLaunch;
use directories::ProjectDirs;

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

    /// Creates a new Helix instance
    Create,

    /// Lists  Helix instances
    List,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = HelixLauncher::parse();
    pretty_env_logger::formatted_builder()
        .filter_level(cli.verbosity.log_level_filter())
        .init();
    
    match cli.subcommand {
        Command::Start => {}
        Command::Create => { create_instance().await?; }
        Command::List => { list_instances().await?; }
    }
    
    Ok(())
}

async fn create_instance() -> Result<()> {
    
    // creation wizard
    // probably a better way to do this
    
    println!("Enter instance name: ");
    let mut name = String::new();
    io::stdin().read_line(&mut name).expect("error: unable to read user input");
    name = name.trim().to_owned();

    println!("Enter minecraft version: ");
    let mut version = String::new();
    io::stdin().read_line(&mut version).expect("error: unable to read user input");
    version = version.trim().to_owned();
    
    // add modloader later? 
        
    let project_dir = ProjectDirs::from("dev", "HelixLauncher", "hxmc").unwrap();
    let instances_path = project_dir.data_dir().join("instances");

    fs::create_dir_all(&instances_path)?;
    
    Instance::new(name, version, InstanceLaunch::default(), &instances_path);

    Ok(())
}
async fn list_instances() -> Result<()> {

    let project_dir = ProjectDirs::from("dev", "HelixLauncher", "hxmc").unwrap();
    let instances_path = project_dir.data_dir().join("instances");

    let instances = Instance::list_instances(instances_path);

    for i in instances {
        println!("Instance: {}", i.name);
    }

    Ok(())
}