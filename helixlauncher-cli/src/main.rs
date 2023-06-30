//! Helix Launcher CLI
//! This is an example implementation of the Helix Launcher CLI.

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use helixlauncher_core::auth::account::AccountConfig;
use helixlauncher_core::auth::{MinecraftAuthenticator, DEFAULT_ACCOUNT_JSON};
use helixlauncher_core::config::Config;
use helixlauncher_core::launch::{
    asset::merge_components,
    instance::{Instance, InstanceLaunchConfig, Modloader},
    prepared::{prepare_launch, LaunchOptions},
};

#[derive(Parser, Debug)]
struct HelixLauncher {
    #[command(subcommand)]
    subcommand: Command,

    #[command(flatten)]
    verbosity: Verbosity<InfoLevel>,
}

#[derive(Debug, ValueEnum, Clone)]
enum ClapModloader {
    Fabric,
    Forge,
    Quilt,
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
    Create {
        #[arg(requires = "version")]
        name: Option<String>,
        #[arg(long, requires = "name")]
        version: Option<String>,
        #[arg(long, requires = "name")]
        modloader: Option<ClapModloader>,
        #[arg(long, requires = "modloader")]
        modloader_version: Option<String>,
    },

    /// Lists instances
    List,

    /// Lists accounts
    AccountList,

    /// Add new account
    AccountNew,

    /// Select an account
    AccountSelect,
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
        Command::Create {
            name,
            version,
            modloader,
            modloader_version,
        } => {
            create_instance(&config, name, version, modloader, modloader_version).await?;
        }
        Command::List => {
            list_instances(&config).await?;
        }
        Command::AccountList => {
            get_accounts_cmd(&config).await?;
        }
        Command::AccountNew => {
            add_account_cmd(&config).await?;
        }
        Command::AccountSelect => {
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

    let account_config =
        AccountConfig::new(config.get_base_path().as_path().join(DEFAULT_ACCOUNT_JSON))?;
    let account = account_config.default.and_then(|selected| {
        account_config
            .accounts
            .into_iter()
            .find(|it| it.uuid == selected)
    });
    let prepared = prepare_launch(
        config,
        &instance,
        &components,
        LaunchOptions::default().world(world).account(account),
    )
    .await?;
    if !dry_run {
        prepared.launch(true).await?.wait().await?;
    } else {
        println!("{:?}", prepared);
    }
    Ok(())
}

async fn create_instance(
    config: &Config,
    name: Option<String>,
    version: Option<String>,
    modloader: Option<ClapModloader>,
    modloader_version: Option<String>,
) -> Result<()> {
    // creation wizard
    let (name, version, modloader, modloader_version) = if let Some(name) = name {
        let version = version.unwrap(); // required in clap
        let (modloader, modloader_version) = if let Some(modloader) = modloader {
            let modloader = match modloader {
                ClapModloader::Fabric => Modloader::Fabric,
                ClapModloader::Forge => Modloader::Forge,
                ClapModloader::Quilt => Modloader::Quilt,
            };
            let modloader_version = if let Some(modloader_version) = modloader_version {
                modloader_version
            } else {
                todo!("Modloader version required")
            };
            (modloader, Some(modloader_version))
        } else {
            (Modloader::Vanilla, None)
        };
        (name, version, modloader, modloader_version)
    } else {
        let name = inquire::Text::new("Instance name").prompt()?;
        let version = inquire::Text::new("Minecraft version").prompt()?;
        let modloader = inquire::Select::new(
            "Modloader",
            vec![
                Modloader::Vanilla,
                Modloader::Quilt,
                Modloader::Forge,
                Modloader::Fabric,
            ],
        )
        .prompt()?;

        let modloader_version = if modloader != Modloader::Vanilla {
            // TODO: should we mention the loader in this prompt?
            Some(inquire::Text::new("Modloader version").prompt()?)
        } else {
            None
        };
        (name, version, modloader, modloader_version)
    };

    let instance = Instance::new(
        name,
        version,
        InstanceLaunchConfig::default(),
        &config.get_instances_path(),
        modloader,
        modloader_version,
    )?;
    println!("Instance \"{}\" created!", instance.config.name);
    Ok(())
}
async fn list_instances(config: &Config) -> Result<()> {
    let instances = Instance::list_instances(config.get_instances_path())?;

    for i in instances {
        println!("Instance: {}", i.config.name);
    }

    Ok(())
}

fn add_account_callback(code: String, uri: String, message: String) {
    println!("code: {}", code);
    println!("uri: {}", uri);
    println!("message: {}", message);
}

async fn add_account_cmd(config: &Config) -> Result<()> {
    let minecraft_authenticator: MinecraftAuthenticator =
        MinecraftAuthenticator::new("1d644380-5a23-4a84-89c3-5d29615fbac2");
    let account = minecraft_authenticator
        .initial_auth(add_account_callback)
        .await?;
    let username = account.username.clone();
    let mut account_config =
        AccountConfig::new(config.get_base_path().as_path().join(DEFAULT_ACCOUNT_JSON))?;
    let stored_account = account_config
        .accounts
        .iter_mut()
        .find(|it| it.uuid == account.uuid);
    match stored_account {
        None => {
            if account_config.accounts.len() == 0 {
                account_config.default = Some(account.uuid.clone())
            }
            account_config.accounts.push(account);
        }
        Some(stored_account) => {
            stored_account.refresh_token = account.refresh_token;
            stored_account.username = account.username;
            stored_account.token = account.token;
        }
    }
    account_config.save()?;
    println!("Welcome! You are logged in as: {}", username);
    Ok(())
}

async fn get_accounts_cmd(config: &Config) -> Result<()> {
    let account_config =
        AccountConfig::new(config.get_base_path().as_path().join(DEFAULT_ACCOUNT_JSON))?;
    for account in account_config.accounts {
        println!("- {}", account.username);
    }
    Ok(())
}
