use std::{collections::HashMap, fs::File, io, path::PathBuf, process::Stdio};

use anyhow::Result;
use futures::stream::{self, StreamExt, TryStreamExt};
use helixlauncher_meta::component::{self, Hash, MinecraftArgument};

use lazy_static::lazy_static;
use regex::{Captures, Regex};
use tokio::{
    fs,
    process::{Child, Command},
    task,
};

use crate::{
    auth::account::Account,
    config::Config,
    fsutil::{check_path, copy_file},
};

use super::{
    asset::MergedComponents,
    asset::{Asset, AssetIndex},
    download_file, generate_classpath, instance, LaunchError,
};

#[derive(Debug)]
pub struct PreparedLaunch {
    pub working_directory: PathBuf,
    pub java_path: String,
    pub jvm_args: Vec<String>,
    pub classpath: Vec<String>,
    pub main_class: String,
    pub args: Vec<String>,
}

impl PreparedLaunch {
    // TODO: add better API for log output
    pub async fn launch(&self, inherit_out: bool) -> Result<Child, LaunchError> {
        if !inherit_out {
            todo!();
        }
        let classpath = generate_classpath(&self.classpath);
        // TODO: hook up javalaunch
        Ok(Command::new(&self.java_path)
            .current_dir(&self.working_directory)
            .args(&self.jvm_args)
            .arg("-classpath")
            .arg(classpath)
            .arg(&self.main_class)
            .args(&self.args)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?)
    }
}

#[derive(Debug, Default)]
#[non_exhaustive]
pub struct LaunchOptions<'a> {
    world: Option<String>,
    account: Option<&'a Account>, // TODO: should this be a reference?
}

impl LaunchOptions<'_> {
    pub fn world(self, world: Option<String>) -> Self {
        Self { world, ..self }
    }

    pub fn account(self, account: Option<&Account>) -> LaunchOptions<'_> {
        LaunchOptions { account, ..self }
    }

    pub fn has_world(&self) -> bool {
        self.world.is_some()
    }

    pub fn account_or_default(&self) -> (String, String, String) {
        if let Some(account) = self.account {
            (
                account.username.to_string(),
                account.uuid.to_string(),
                account.token.to_string(),
            )
        } else {
            (
                "Player".to_string(),
                "00000000-0000-0000-0000-000000000000".to_string(),
                String::new(),
            )
        }
    }
}

pub async fn prepare_launch(
    config: &Config,
    instance: &instance::Instance,
    components: &MergedComponents,
    launch_options: LaunchOptions<'_>,
) -> Result<PreparedLaunch> {
    // TODO: global default config
    let java_path = instance
        .config
        .launch
        .javapath
        .as_ref()
        .map_or("java", |s| s)
        .to_string(); // TODO
    let game_dir = instance.get_game_dir();
    let natives_path = instance.path.join("natives");

    if launch_options.has_world() && !components.has_trait(component::Trait::SupportsQuickPlayWorld)
    {
        return Err(LaunchError::UnsupportedFeature {
            name: String::from("Launching into world"),
        })?;
    }

    // Set locale to English to make Java's String.toUpperCase/toLowerCase return predictable results if mods forgot to pass a locale
    let mut jvm_args = vec![
        String::from("-Duser.language=en"),
        format!("-Djava.library.path={}", natives_path.to_str().unwrap()),
    ];

    if let Some(allocation) = &instance.config.launch.allocation {
        jvm_args.append(&mut vec![
            format!("-Xms{}M", allocation.min),
            format!("-Xmx{}M", allocation.max),
        ]);
    }

    if let Some(instance_jvm_args) = &instance.config.launch.jvm_args {
        jvm_args.append(&mut instance_jvm_args.clone());
    }

    let mut args = vec![];

    for argument in &components.arguments {
        let arg = match argument {
            MinecraftArgument::Always(arg) => arg,
            MinecraftArgument::Conditional { value, feature } => {
                if !match feature {
                    component::ConditionFeature::Demo => launch_options.account.is_none(),
                    component::ConditionFeature::QuickPlayWorld => launch_options.world.is_some(),
                    _ => false, // TODO
                } {
                    continue;
                }
                value
            }
        };

        args.push(arg);
    }

    let (username, uuid, token) = launch_options.account_or_default();

    let mut props = HashMap::new();
    props.insert("user.name", username.as_str());
    props.insert("user.uuid", uuid.as_str());
    props.insert("user.token", token.as_str());
    props.insert("user.type", "msa");
    props.insert("instance.game_dir", game_dir.to_str().unwrap());

    if let Some(minecraft_version) = instance.get_component_version("net.minecraft") {
        props.insert("instance.minecraft_version", minecraft_version);
    }

    if let Some(world) = &launch_options.world {
        props.insert("launch.world", world);
    }

    let paths = components.get_all(config, instance).await?;

    let game_jar = components.get_jar(&paths, &game_dir)?;

    let mut classpath = vec![game_jar.into_os_string().into_string().unwrap()];

    components
        .classpath
        .clone()
        .into_iter()
        .for_each(|entry| classpath.push(paths[&entry].to_str().unwrap().to_string()));

    let assets_dir; // outside the block, so that it outlives the block for props
    let unpack_path;

    let client = reqwest::Client::new();

    if let Some(assets) = &components.assets {
        assets_dir = config.get_assets_path();
        props.insert("instance.assets_dir", assets_dir.to_str().unwrap());

        props.insert("instance.assets_index_name", &assets.id);
        let mut index_path = assets_dir.join("indexes");
        index_path.push(format!("{}.json", assets.id));

        download_file(
            &client,
            &assets_dir
                .join("indexes")
                .join(format!("{}.json", assets.id)),
            &assets.url,
            assets.size,
            &Hash::SHA1(assets.sha1.to_string()),
        )
        .await?;

        let index: AssetIndex = serde_json::from_slice(&fs::read(index_path).await?)?;

        unpack_path = if index.map_to_resources {
            Some(game_dir.join("resources"))
        } else if index.r#virtual {
            let mut virtual_dir = game_dir.join("assets");
            virtual_dir.push("virtual");
            virtual_dir.push(&assets.id);
            Some(virtual_dir)
        } else {
            None
        };

        if let Some(unpack_path) = &unpack_path {
            props.insert("instance.virtual_assets_dir", unpack_path.to_str().unwrap());
        }

        stream::iter(index.objects)
            .map(Ok)
            .try_for_each_concurrent(16, |(name, Asset { hash, size })| {
                let client = client.clone();
                let assets_dir = assets_dir.clone();
                let unpack_path = unpack_path.clone();

                async move {
                    let hash_part = &hash[..2];
                    let mut asset_path = assets_dir.join("objects");
                    asset_path.push(hash_part);
                    asset_path.push(&hash);
                    download_file(
                        &client,
                        &asset_path,
                        &format!("https://resources.download.minecraft.net/{hash_part}/{hash}"),
                        size,
                        &Hash::SHA1(hash),
                    )
                    .await?;
                    if let Some(unpack_path) = unpack_path {
                        if !check_path(&name) {
                            return Err(LaunchError::InvalidFilename {
                                name: name.to_string(),
                            })?;
                        }
                        let unpack_file = unpack_path.join(name);
                        fs::create_dir_all(unpack_file.parent().unwrap()).await?;
                        copy_file(&asset_path, &unpack_file)?;
                    }
                    anyhow::Ok(())
                }
            })
            .await?;
    }

    for native in &components.natives {
        let native = native.clone();
        let file_path = paths[&native.name].clone();
        let natives_path = natives_path.clone();
        task::spawn_blocking(move || {
            let mut zip = zip::ZipArchive::new(File::open(file_path)?)?;
            for i in 0..zip.len() {
                // TODO: are ZIP bombs an issue here? if this code gets invoked, code from the
                // instance and components is about to get executed anyways
                let mut entry = zip.by_index(i)?;
                if !entry.is_file() {
                    continue;
                }
                let name = entry.name().to_string(); // need to copy, otherwise entry is immutably
                                                     // borrowed, preventing the read below
                if native
                    .exclusions
                    .iter()
                    .any(|exclusion| name.starts_with(exclusion))
                {
                    continue;
                }
                if !check_path(&name) {
                    return Err(LaunchError::InvalidFilename { name })?;
                }
                let path = natives_path.join(name);
                std::fs::create_dir_all(path.parent().unwrap())?; // unwrap is safe here, at minimum
                                                                  // there will be the natives folder
                io::copy(&mut entry, &mut File::create(path)?)?;
            }
            anyhow::Ok(())
        })
        .await
        .unwrap()?; // the unwrap here triggers when the inner closure has panicked
    }

    lazy_static! {
        static ref VAR_PATTERN: Regex = Regex::new(r"\$\{([a-zA-Z0-9_.]+)\}").unwrap();
    }

    Ok(PreparedLaunch {
        java_path,
        jvm_args,
        classpath,
        main_class: components.main_class.clone(),
        args: args
            .into_iter()
            .map(|arg| {
                VAR_PATTERN
                    .replace_all(arg, |captures: &Captures<'_>| {
                        println!("{captures:?}");
                        props[captures.get(1).unwrap().as_str()]
                    })
                    .into_owned()
            })
            .collect(),
        working_directory: game_dir,
    })
}

/*pub async fn mc_version_exists(version: String) -> bool {
    let response = reqwest::get(format!("{META}net.minecraft/index.json")).await.expect("Meta server not found"); // TODO don't hardcode meta maybe?
    let index: MinecraftIndexResponse = response.json().await.unwrap();
    let mut found: bool = false;
    for x in index.items {
        if x.version == version {
            found = true;
        }
    }
    found
}*/
