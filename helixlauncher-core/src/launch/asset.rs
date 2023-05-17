use std::{
    collections::{HashMap, BTreeSet, HashSet},
    borrow::{Borrow, Cow},
    path::PathBuf, fs::File
};

use anyhow::Result;
use helixlauncher_meta::{util::{GradleSpecifier, CURRENT_ARCH, CURRENT_OS}, component::{self, MinecraftArgument, Hash, ConditionalClasspathEntry, Platform}};
use indexmap::IndexMap;
use regex::Regex;
use serde::Deserialize;

use lazy_static::lazy_static;

use crate::config::Config;

use super::{instance::{self, Instance}, download_file};

#[derive(Debug)]
pub struct MergedComponents {
    pub classpath: Vec<GradleSpecifier>,
    pub natives: Vec<Native>,
    pub artifacts: HashMap<GradleSpecifier, Artifact>,
    pub traits: BTreeSet<component::Trait>,
    pub assets: Option<component::Assets>,
    pub game_jar: GradleSpecifier,
    pub jarmods: Vec<GradleSpecifier>,
    pub main_class: String,
    pub arguments: Vec<MinecraftArgument>,
}

impl MergedComponents {
    pub fn has_trait(&self, check: component::Trait) -> bool {
        self.traits.contains(&check)
    }

    // TODO: parallelize
    pub async fn get_all(&self, config: &Config, instance: &Instance) -> Result<HashMap<&GradleSpecifier, PathBuf>> {
        let mut needed_artifacts = HashMap::with_capacity(self.artifacts.len());

        for library in &self.classpath {
            needed_artifacts
                .entry(library)
                .or_insert(&self.artifacts[library]);
        }

        needed_artifacts
            .entry(&self.game_jar)
            .or_insert(&self.artifacts[&self.game_jar]);

        for jarmod in &self.jarmods {
            needed_artifacts
                .entry(jarmod)
                .or_insert(&self.artifacts[jarmod]);
        }

        for native in &self.natives {
            needed_artifacts
                .entry(&native.name)
                .or_insert(&self.artifacts[&native.name]);
        }

        let mut paths = HashMap::with_capacity(needed_artifacts.len());

        // TODO: this may need some ordering for artifacts with processing dependencies
        // TODO: temporary files for "atomic" writes?
        let client = reqwest::Client::new();
        for (name, artifact) in needed_artifacts.into_iter() {

            paths.insert(name, artifact.get(name, &client, config, instance).await?);
        }

        return Ok(paths);
    }

    pub fn get_jar(&self, paths: &HashMap<&GradleSpecifier, PathBuf>, game_dir: &PathBuf) -> Result<PathBuf> {
        if self.jarmods.is_empty() {
            return Ok(paths[&self.game_jar].clone())
        }
        let mut minecraft_jar = game_dir.join("bin");
        minecraft_jar.push("minecraft.jar");
        let mut zip_writer = zip::ZipWriter::new(File::create(&minecraft_jar)?);
        let mut written_files = HashSet::new();
        for jarmod in &self.jarmods {
            let file = &paths[jarmod];
            let mut zip = zip::ZipArchive::new(File::open(file)?)?;
            for i in 0..zip.len() {
                let file = zip.by_index_raw(i)?;
                if written_files.insert(file.name().to_string()) {
                    zip_writer.raw_copy_file(file)?;
                }
            }
        }
        Ok(minecraft_jar)
    }
}

#[derive(Debug)]
pub struct Native {
    pub name: GradleSpecifier,
    pub exclusions: Vec<String>,
}

#[derive(Debug)]
pub enum Artifact {
    Download { url: String, size: u32, hash: Hash },
}

impl Artifact {
    pub fn clean_name(name: &str) -> Cow<'_, str> {
        lazy_static! {
            static ref CLEAN_NAME_REGEX: Regex = Regex::new(r"[^a-zA-Z0-9.\-_]|^\.").unwrap();
        }
        CLEAN_NAME_REGEX.replace_all(name, "__")
    }

    pub fn get_path(
        &self,
        name: &GradleSpecifier,
        config: &Config,
        _instance: &Instance
    ) -> PathBuf {
        match self {
            Self::Download {
                url: _,
                size: _,
                hash: _,
            } => {
                let mut path = config.get_libraries_path();
                for part in name.group.split('.') {
                    path.push::<&str>(&Self::clean_name(part));
                }
                path.push::<&str>(&Self::clean_name(&name.artifact));
                path.push::<&str>(&Self::clean_name(&name.version));
                path.push::<&str>(
                    Self::clean_name(&format!(
                        "{}-{}{}.{}",
                        name.artifact,
                        name.version,
                        if let Some(ref classifier) = &name.classifier {
                            format!("-{}", classifier)
                        } else {
                            String::new()
                        },
                        name.extension
                    ))
                    .borrow(),
                );
                path
            }
        }
    }

    pub async fn get(&self, name: &GradleSpecifier, client: &reqwest::Client, config: &Config, instance: &Instance) -> Result<PathBuf> {
        let value = match self {
            Artifact::Download { url, size, hash } => {
                let path = self.get_path(name, config, instance);
                download_file(client, &path, url, *size, hash).await?;
                path
            }
        };

        return Ok(value);
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetIndex {
    #[serde(default)]
    pub map_to_resources: bool,
    #[serde(default)]
    pub r#virtual: bool,
    #[serde()]
    pub objects: HashMap<String, Asset>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Asset {
    pub hash: String,
    pub size: u32,
}

// TODO: proper error (and progress?) handling
// TODO: this doesn't handle stuff like Rosetta or running x86 Java on x86_64 at all
pub async fn merge_components(
    config: &Config,
    components: &Vec<instance::Component>,
) -> Result<MergedComponents> {
    let mut classpath = IndexMap::new();
    let mut jarmods = IndexMap::new();
    let mut traits = BTreeSet::new();
    let mut artifacts = HashMap::new();
    let mut natives = vec![];
    let mut game_jar = None;
    let mut assets = None;
    let mut main_class = None;
    let mut arguments = vec![];

    for component in components {
        let mut meta = component.into_meta(config).await?;
        for trait_ in meta.traits {
            traits.insert(trait_);
        }

        for native in meta.natives {
            if platform_matches(native.platform) {
                natives.push(Native {
                    name: native.name,
                    exclusions: native.exclusions,
                });
            }
        }

        // TODO: should we include jarmods after a game jar was defined?
        if game_jar.is_none() {
            for jarmod in meta.jarmods {
                let unversioned_name = GradleSpecifier {
                    version: String::from(""),
                    ..jarmod.clone()
                };
                jarmods.entry(unversioned_name).or_insert(jarmod);
            }
        }

        for classpath_entry in meta.classpath {
            let name = match classpath_entry {
                ConditionalClasspathEntry::All(name) => name,
                ConditionalClasspathEntry::PlatformSpecific { name, platform } => {
                    if !platform_matches(platform) {
                        continue;
                    }
                    name
                }
            };
            let unversioned_name = GradleSpecifier {
                version: String::from(""),
                ..name.clone()
            };
            classpath.entry(unversioned_name).or_insert(name);
        }

        for download in meta.downloads {
            artifacts
                .entry(download.name)
                .or_insert(Artifact::Download {
                    url: download.url,
                    size: download.size,
                    hash: download.hash,
                });
        }

        game_jar = game_jar.or(meta.game_jar);
        assets = assets.or(meta.assets);
        main_class = main_class.or(meta.main_class);

        // `meta` is going out of scope immediately after, so we don't need to worry about it clearing.
        arguments.append(&mut meta.game_arguments);
    }

    Ok(MergedComponents {
        classpath: classpath.into_values().collect(),
        natives,
        artifacts,
        traits,
        game_jar: game_jar.unwrap(),
        assets,
        jarmods: jarmods.into_values().collect(),
        main_class: main_class.unwrap(),
        arguments,
    })
}

fn platform_matches(platform: Platform) -> bool {
    if let Some(arch) = platform.arch && arch != CURRENT_ARCH {
        return false;
    }
    if !platform.os.is_empty() && !platform.os.contains(&CURRENT_OS) {
        return false;
    }
    true
}
