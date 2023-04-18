//! Helix Launcher CLI
//! This is an example implementation of the Helix Launcher CLI.

use std::io;

use anyhow::Result;
use clap::{Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use helixlauncher_core::auth::account::{add_account, get_accounts, Account};
use helixlauncher_core::auth::{MinecraftAuthenticator, DEFAULT_ACCOUNT_JSON};
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
    Create {
        #[arg(long)]
        name: Option<String>,
        #[arg(long, alias("version"))]
        mc_version: Option<String>,
        #[arg(long)]
        modloader: Option<String>,
        #[arg(long)]
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
            mc_version,
            modloader,
            modloader_version,
        } => {
            create_instance(&config, name, mc_version, modloader, modloader_version).await?;
        }
        Command::List => {
            list_instances(&config).await?;
        }
        Command::AccountList => {
            get_accounts_cmd(&config).await;
        }
        Command::AccountNew => {
            add_account_cmd(&config).await;
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

    let accounts = get_accounts(
        config
            .get_base_path()
            .as_path()
            .join(DEFAULT_ACCOUNT_JSON)
            .as_path(),
    )?;
    let mut account: Option<Account> = None;
    for a in accounts {
        if a.selected {
            account = Some(a)
        }
    }
    let prepared = prepare_launch(
        config,
        &instance,
        &components,
        LaunchOptions::default().world(world).account(account),
    )
    .await?;
    if !dry_run {
        launch(&prepared, true).await?.wait().await?;
    } else {
        println!("{:?}", prepared);
    }
    Ok(())
}

async fn create_instance(
    config: &Config,
    name_arg: Option<String>,
    mc_version_arg: Option<String>,
    modloader_arg: Option<String>,
    modloader_version_arg: Option<String>,
) -> Result<()> {
    // creation wizard
    // probably a better way to do this - probably even more so for modloader bit

    let mut name = String::new();
    if let Some(sname_arg) = name_arg {
        name = sname_arg.to_owned();
    } else {
        println!("Enter instance name: ");
        io::stdin()
            .read_line(&mut name)
            .expect("error: unable to read user input");
        name = name.trim().to_owned();
    }

    let mut version = String::new();
    if let Some(smc_version_arg) = mc_version_arg {
        version = smc_version_arg.to_owned();
    } else {
        println!("Enter minecraft version: ");
        io::stdin()
            .read_line(&mut version)
            .expect("error: unable to read user input");
        version = version.trim().to_owned();
    }

    let mut modloader_string = String::new();
    if let Some(smodloader_arg) = modloader_arg {
        modloader_string = smodloader_arg.to_owned();
    } else {
        println!("Enter modloader: ");
        io::stdin()
            .read_line(&mut modloader_string)
            .expect("error: unable to read user input");
        modloader_string = modloader_string.trim().to_lowercase();
    }
    let modloader = match &*modloader_string {
        "quilt" | "quiltmc" => Modloader::Quilt,
        "fabric" | "fabricmc" => Modloader::Fabric,
        "forge" | "minecraftforge" => Modloader::Forge,
        "vanilla" => Modloader::Vanilla,
        _ => {
            println!("warn: using vanilla for modloader as input is invalid");
            Modloader::Vanilla
        }
    };

    let modloader_version = if matches!(
        modloader,
        Modloader::Quilt | Modloader::Fabric | Modloader::Forge
    ) {
        let mut output = Some(String::new());
        if let Some(smodloader_version_arg) = modloader_version_arg {
            output = Some(smodloader_version_arg.trim().to_owned());
        } else {
            println!("Enter modloader version: ");
            let mut modloader_version = String::new();
            io::stdin()
                .read_line(&mut modloader_version)
                .expect("error: unable to read user input");
            output = Some(modloader_version.trim().to_owned());
        }
        output
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
    println!("Instance \"{}\" created!", name);
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

async fn add_account_cmd(config: &Config) {
    let minecraft_authenticator: MinecraftAuthenticator =
        MinecraftAuthenticator::new("1d644380-5a23-4a84-89c3-5d29615fbac2");
    let out = minecraft_authenticator
        .initial_auth(add_account_callback)
        .await;
    if let Ok(account) = out {
        let username = account.username.clone();
        let uuid = account.uuid.clone();
        let e_out = get_accounts(
            config
                .get_base_path()
                .as_path()
                .join(DEFAULT_ACCOUNT_JSON)
                .as_path(),
        );
        let mut exists = false;
        if let Ok(accounts) = e_out {
            for acc in accounts {
                if uuid == acc.uuid {
                    exists = true;
                }
            }
        }
        let mut no_print = false;
        if !exists {
            let add_acc_res = add_account(
                account,
                config
                    .get_base_path()
                    .as_path()
                    .join(DEFAULT_ACCOUNT_JSON)
                    .as_path(),
            );
            if add_acc_res.is_err() {
                no_print = true;
            }
        }
        if !no_print {
            println!("Welcome! You are logged in as: {}", username);
        }
    }
}

async fn get_accounts_cmd(config: &Config) {
    let out = get_accounts(
        config
            .get_base_path()
            .as_path()
            .join(DEFAULT_ACCOUNT_JSON)
            .as_path(),
    );
    if out.is_ok() {
        let accounts = out.unwrap();
        for account in accounts {
            println!("- {}", account.username);
        }
    }
}
