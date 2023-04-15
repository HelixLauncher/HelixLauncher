//! Helix Launcher CLI
//! This is an example implementation of the Helix Launcher CLI.

use std::io;

use anyhow::{Ok, Result};
use clap::{Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use helixlauncher_core::config::Config;
use helixlauncher_core::game::{merge_components, prepare_launch, LaunchOptions};
use helixlauncher_core::instance::{Instance, InstanceLaunch, Modloader};
use helixlauncher_core::launcher::launch;

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
    Launch {
        name: String,
        #[arg(long)]
        world: Option<String>,
        #[arg(long, short = 'n')]
        dry_run: bool,
    },

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
    let config = Config::new("dev.helixlauncher.HelixLauncher", "HelixLauncher")?;
    pretty_env_logger::formatted_builder()
        .filter_level(cli.verbosity.log_level_filter())
        .init();

    match cli.subcommand {
        Command::Launch {
            name,
            world,
            dry_run,
        } => {
            launch_instance(&config, name, world, dry_run).await?;
        }
        Command::Create => {
            create_instance(&config).await?;
        }
        Command::List => {
            list_instances(&config).await?;
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

async fn launch_instance(
    config: &Config,
    name: String,
    world: Option<String>,
    dry_run: bool,
) -> Result<()> {
    let instance = Instance::from_path(config.get_instances_path().join(name))?;
    let components = merge_components(config, &instance.config.components).await?;
    let prepared = prepare_launch(
        config,
        &instance,
        &components,
        LaunchOptions::default().world(world),
    )
    .await?;
    if !dry_run {
        launch(&prepared, true).await?.wait().await?;
    } else {
        println!("{:?}", prepared);
    }
    Ok(())
}

async fn create_instance(config: &Config) -> Result<()> {
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
    modloader_string = modloader_string.trim().to_lowercase();

    let modloader = match &*modloader_string {
        "quilt" | "quiltmc" => Modloader::Quilt,
        "fabric" | "fabricmc" => Modloader::Fabric,
        "forge" | "minecraftforge" => Modloader::Forge,
        "vanilla" => Modloader::Vanilla,
        _ => {
            println!("warn: using vanilla as modloader is invalid");
            Modloader::Vanilla
        }
    };

    let modloader_version = if matches!(
        modloader,
        Modloader::Quilt | Modloader::Fabric | Modloader::Forge
    ) {
        println!("Enter modloader version: ");
        let mut modloader_version = String::new();
        io::stdin()
            .read_line(&mut modloader_version)
            .expect("error: unable to read user input");
        Some(modloader_version.trim().to_owned())
    } else {
        None
    };

    Instance::new(
        name,
        version,
        InstanceLaunch::default(),
        &config.get_instances_path(),
        modloader,
        modloader_version,
    )?;

    Ok(())
}
async fn list_instances(config: &Config) -> Result<()> {
    let instances = Instance::list_instances(config.get_instances_path())?;

    for i in instances {
        println!("Instance: {}", i.config.name);
    }

    Ok(())
}
