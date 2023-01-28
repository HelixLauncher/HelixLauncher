//! Helix Launcher CLI
//! This is an example implementation of the Helix Launcher CLI.

use std::fs;
use std::io;
use std::io::Read;

use anyhow::{Ok, Result};
use clap::{Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use directories::ProjectDirs;
use helixlauncher_core::instance::Instance;
use helixlauncher_core::instance::InstanceLaunch;
use helixlauncher_core::instance::Modloader;

#[derive(Parser, Debug)]
struct HelixLauncher {
    #[command(subcommand)]
    subcommand: Command,

    #[command(flatten)]
    verbosity: Verbosity<InfoLevel>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Launches a new instance
    Start,

    /// Creates a new instance
    Create,

    /// Lists instances
    List,

    /// Lists accounts
    AccountList,

    /// Add new account
    AccountNew,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = HelixLauncher::parse();
    pretty_env_logger::formatted_builder()
        .filter_level(cli.verbosity.log_level_filter())
        .init();

    match cli.subcommand {
        Command::Start => {}
        Command::Create => {
            create_instance().await?;
        }
        Command::List => {
            list_instances().await?;
        }
        Command::AccountList => {
            todo!();
        }
        Command::AccountNew => {
            todo!();
        }
    }

    Ok(())
}

async fn create_instance() -> Result<()> {
    // creation wizard
    // probably a better way to do this - probably even more so for modloader bit

    println!("Enter instance name: ");
    let mut name = String::new();
    io::stdin()
        .read_line(&mut name)
        .expect("error: unable to read user input");
    name = name.trim().to_owned();

    println!("Enter minecraft version: ");
    let mut version = String::new();
    io::stdin()
        .read_line(&mut version)
        .expect("error: unable to read user input");
    version = version.trim().to_owned();

    println!("Enter modloader: ");
    let mut modloader_string = String::new();
    io::stdin()
        .read_line(&mut modloader_string)
        .expect("error: unable to read user input");
    modloader_string = modloader_string.trim().to_lowercase().to_owned();
    let modloader_str = modloader_string.as_str();
    let modloader = match modloader_str {
        "quilt" | "quiltmc" => Modloader::Quilt,
        "fabric" | "fabricmc" => Modloader::Fabric,
        "forge" | "minecraftforge" => Modloader::Forge,
        _ => Modloader::Vanilla,
    };
    if (modloader_str != "quilt"
        && modloader_str != "quiltmc"
        && modloader_str != "fabric"
        && modloader_str != "fabricmc"
        && modloader_str != "forge"
        && modloader_str != "minecraftforge"
        && modloader_str != "none"
        && modloader_str != "vanilla")
    {
        println!("warn: using vanilla as modloader is invalid")
    }

    let mut modloader_version = String::from("invalid_modloader_version_0x1");
    
    if matches!(modloader, Modloader::Quilt)
        || matches!(modloader, Modloader::Fabric)
        || matches!(modloader, Modloader::Forge)
    {
        println!("Enter modloader version: ");
        io::stdin()
            .read_line(&mut modloader_version)
            .expect("error: unable to read user input");
        modloader_version = modloader_version.trim().to_owned().replace("invalid_modloader_version_0x1", "");
    }

    let project_dir = ProjectDirs::from("dev", "HelixLauncher", "hxmc").unwrap();
    let instances_path = project_dir.data_dir().join("instances");

    fs::create_dir_all(&instances_path)?;
    Instance::new(
        name,
        version,
        InstanceLaunch::default(),
        &instances_path,
        modloader,
        match modloader_version.as_str() {
            "invalid_modloader_version_0x1" => None,
            _ => Some(modloader_version),
        },
    )?;

    Ok(())
}
async fn list_instances() -> Result<()> {
    let project_dir = ProjectDirs::from("dev", "HelixLauncher", "hxmc").unwrap();
    let instances_path = project_dir.data_dir().join("instances");

    let instances = Instance::list_instances(instances_path)?;

    for i in instances {
        println!("Instance: {}", i.name);
    }

    Ok(())
}
