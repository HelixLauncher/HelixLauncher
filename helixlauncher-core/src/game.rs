use std::{
    borrow::{Borrow, Cow},
    collections::{HashMap, HashSet},
    error::Error,
    fs::{self, File},
    io::{self, Read},
    path::{Path, PathBuf},
};

use digest::Digest;
use helixlauncher_meta::{
    component::{self, Component, ConditionalClasspathEntry, Hash, Platform},
    index::Index,
    util::{GradleSpecifier, CURRENT_ARCH, CURRENT_OS},
};

use hex::ToHex;
use indexmap::IndexMap;
use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;
use thiserror::Error;

use crate::{
    config::Config,
    instance,
    util::{check_path, copy_file},
};

const META: &str = "https://meta.helixlauncher.dev/";

#[derive(Debug)]
pub struct MergedComponents {
    pub classpath: Vec<GradleSpecifier>,
    pub natives: Vec<Native>,
    pub artifacts: HashMap<GradleSpecifier, Artifact>,
    pub traits: Vec<component::Trait>,
    pub assets: Option<component::Assets>,
    pub game_jar: GradleSpecifier,
    pub jarmods: Vec<GradleSpecifier>,
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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AssetIndex {
    #[serde(default)]
    pub map_to_resources: bool,
    #[serde(default)]
    pub r#virtual: bool,
    #[serde()]
    pub objects: HashMap<String, Asset>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Asset {
    pub hash: String,
    pub size: u32,
}

#[derive(Debug, Error)]
pub enum PrepareError {
    #[error("Download of {url} failed: expected file with {expected_hash} and size {expected_size}, found file with hash {actual_hash} and size {actual_size}")]
    InvalidFile {
        url: String,
        expected_hash: component::Hash,
        expected_size: u32,
        actual_hash: String,
        actual_size: usize,
    },
    #[error("Invalid filename found: {name}")]
    InvalidFilename { name: String },
}

#[derive(Debug)]
pub struct PreparedLaunch {}

// TODO: proper error (and progress?) handling
// TODO: this doesn't handle stuff like Rosetta or running x86 Java on x86_64 at all
pub async fn merge_components(
    config: &Config,
    components: &Vec<instance::Component>,
) -> Result<MergedComponents, Box<dyn Error>> {
    let mut classpath = IndexMap::new();
    let mut jarmods = IndexMap::new();
    let mut traits = vec![];
    let mut artifacts = HashMap::new();
    let mut natives = vec![];
    let mut game_jar = None;
    let mut assets = None;
    let mut main_class = None;

    for component in components {
        let component = fetch_component(config, &component.id, &component.version).await?;
        for trait_ in component.traits {
            if !traits.contains(&trait_) {
                traits.push(trait_);
            }
        }
        for native in component.natives {
            if platform_matches(native.platform) {
                natives.push(Native {
                    name: native.name,
                    exclusions: native.exclusions,
                });
            }
        }
        // TODO: should we include jarmods after a game jar was defined?
        if game_jar.is_none() {
            for jarmod in component.jarmods {
                let unversioned_name = GradleSpecifier {
                    version: String::from(""),
                    ..jarmod.clone()
                };
                jarmods.entry(unversioned_name).or_insert(jarmod);
            }
        }
        for classpath_entry in component.classpath {
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
        for download in component.downloads {
            artifacts
                .entry(download.name)
                .or_insert(Artifact::Download {
                    url: download.url,
                    size: download.size,
                    hash: download.hash,
                });
        }
        game_jar = game_jar.or(component.game_jar);
        assets = assets.or(component.assets);
        main_class = main_class.or(component.main_class);
    }
    Ok(MergedComponents {
        classpath: classpath.into_values().collect(),
        natives,
        artifacts,
        traits,
        game_jar: game_jar.unwrap(),
        assets,
        jarmods: jarmods.into_values().collect(),
    })
}

fn check_hash(buf: &[u8], hash: &component::Hash) -> (bool, String) {
    let (expected, actual) = match hash {
        Hash::SHA1(hash) => (hash, sha1::Sha1::digest(buf).encode_hex::<String>()),
        Hash::SHA256(hash) => (hash, sha2::Sha256::digest(buf).encode_hex::<String>()),
    };
    (expected == &actual, actual)
}

fn check_file(path: &Path, size: u32, hash: &component::Hash) -> Result<bool, io::Error> {
    // This can be tricked by modifying or deleting the file after or while it is being processed
    // during launch, but let's not consider that an issue.

    // TODO: maybe not read in the entire file at once?
    if !path.try_exists()? {
        return Ok(false);
    }

    let file = match fs::read(path) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(false),
        r => r,
    }?;
    Ok(file.len() == (size as usize) && check_hash(&file, hash).0)
}

async fn download_file(
    path: &Path,
    url: &str,
    size: u32,
    hash: &component::Hash,
) -> Result<(), Box<dyn Error>> {
    if !check_file(path, size, hash)? {
        let data = reqwest::get(url).await?.bytes().await?;
        let (hash_matches, actual_hash) = check_hash(&data, hash);
        if data.len() != size as usize || !hash_matches {
            return Err(PrepareError::InvalidFile {
                url: url.to_string(),
                expected_hash: hash.clone(),
                expected_size: size,
                actual_hash,
                actual_size: data.len(),
            })?;
        }
        fs::write(path, data)?;
    }
    Ok(())
}

pub async fn prepare_launch(
    config: &Config,
    instance: &instance::Instance,
    components: &MergedComponents,
) -> Result<PreparedLaunch, Box<dyn Error>> {
    // TODO: parallelize
    let mut needed_artifacts = HashMap::with_capacity(components.artifacts.len());
    for library in &components.classpath {
        needed_artifacts
            .entry(library)
            .or_insert(&components.artifacts[library]);
    }
    needed_artifacts
        .entry(&components.game_jar)
        .or_insert(&components.artifacts[&components.game_jar]);
    for jarmod in &components.jarmods {
        needed_artifacts
            .entry(jarmod)
            .or_insert(&components.artifacts[jarmod]);
    }
    for native in &components.natives {
        needed_artifacts
            .entry(&native.name)
            .or_insert(&components.artifacts[&native.name]);
    }

    let mut paths = HashMap::with_capacity(needed_artifacts.len());

    // TODO: this may need some ordering for artifacts with processing dependencies
    // TODO: temporary files for "atomic" writes?
    for (name, artifact) in needed_artifacts.into_iter() {
        paths.insert(
            name,
            match artifact {
                Artifact::Download { url, size, hash } => {
                    let path = artifact.get_path(name, config, instance);
                    download_file(&path, url, *size, hash).await?;
                    path
                }
            },
        );
    }

    let game_jar = if !components.jarmods.is_empty() {
        let mut minecraft_jar = instance.path.join(".minecraft");
        minecraft_jar.push("bin");
        minecraft_jar.push("minecraft.jar");
        let mut zip_writer = zip::ZipWriter::new(File::create(&minecraft_jar)?);
        let mut written_files = HashSet::new();
        for jarmod in &components.jarmods {
            let file = &paths[jarmod];
            let mut zip = zip::ZipArchive::new(File::open(file)?)?;
            for i in 0..zip.len() {
                let file = zip.by_index_raw(i)?;
                if written_files.insert(file.name().to_string()) {
                    zip_writer.raw_copy_file(file)?;
                }
            }
        }
        Cow::Owned(minecraft_jar)
    } else {
        Cow::Borrowed(&paths[&components.game_jar])
    };

    if let Some(assets) = &components.assets {
        let assets_dir = config.get_assets_path();
        let mut index_path = assets_dir.join("indexes");
        index_path.push(format!("{}.json", assets.id));
        download_file(
            &assets_dir
                .join("indexes")
                .join(format!("{}.json", assets.id)),
            &assets.url,
            assets.size,
            &Hash::SHA1(assets.sha1.to_string()),
        )
        .await?;
        let index: AssetIndex = serde_json::from_slice(&fs::read(index_path)?)?;
        let unpack_path = if index.map_to_resources {
            Some(instance.path.join("resources"))
        } else if index.r#virtual {
            let mut virtual_dir = instance.path.join("assets");
            virtual_dir.push("virtual");
            virtual_dir.push(&assets.id);
            Some(virtual_dir)
        } else {
            None
        };
        for (name, Asset { hash, size }) in index.objects {
            let hash_part = &hash[..2];
            let mut asset_path = assets_dir.join("objects");
            asset_path.push(hash_part);
            asset_path.push(&hash);
            download_file(
                &asset_path,
                &format!("https://resources.download.minecraft.net/{hash_part}/{hash}"),
                size,
                &Hash::SHA1(hash),
            )
            .await?;
            if let Some(unpack_path) = &unpack_path {
                if !check_path(&name) {
                    return Err(PrepareError::InvalidFilename {
                        name: name.to_string(),
                    })?;
                }
                copy_file(&asset_path, &unpack_path.join(name))?;
            }
        }

        let natives_path = instance.path.join("natives");

        for native in &components.natives {
            let file_path = &paths[&native.name];
            let mut zip = zip::ZipArchive::new(File::open(file_path)?)?;
            'fileloop: for i in 0..zip.len() {
                // TODO: are ZIP bombs an issue here? if this code gets invoked, code from the
                // instance and components is about to get executed anyways
                let mut entry = zip.by_index(i)?;
                if !entry.is_file() {
                    continue;
                }
                let name = entry.name().to_string(); // need to copy, otherwise entry is immutably
                                                     // borrowed, preventing the read below
                for exclusion in &native.exclusions {
                    if name.starts_with(exclusion) {
                        continue 'fileloop;
                    }
                }
                if !check_path(&name) {
                    return Err(PrepareError::InvalidFilename { name })?;
                }
                let path = natives_path.join(name);
                fs::create_dir_all(path.parent().unwrap())?; // unwrap is safe here, at minimum
                                                             // there will be the natives folder
                io::copy(&mut entry, &mut File::create(path)?)?;
            }
        }
    }

    Ok(PreparedLaunch {})
}

impl Artifact {
    fn clean_name(name: &str) -> Cow<str> {
        lazy_static! {
            static ref CLEAN_NAME_REGEX: Regex = Regex::new(r"[^a-zA-Z0-9.\-_]|^\.").unwrap();
        }
        CLEAN_NAME_REGEX.replace_all(name, "__")
    }

    fn get_path(
        &self,
        name: &GradleSpecifier,
        config: &Config,
        instance: &instance::Instance,
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
}

fn platform_matches(platform: Platform) -> bool {
    if let Some(arch) = platform.arch {
        if arch != CURRENT_ARCH {
            return false;
        }
    }
    if !platform.os.is_empty() && !platform.os.contains(&CURRENT_OS) {
        return false;
    }
    true
}

async fn fetch_component(
    config: &Config,
    id: &str,
    version: &str,
) -> Result<Component, Box<dyn Error>> {
    // TODO: better caching
    let component_data_result = async {
        reqwest::get(format!("{META}{id}/{version}.json"))
            .await?
            .bytes()
            .await
    }
    .await;
    let mut path = config.get_base_path().join("meta");
    path.push(id);
    fs::create_dir_all(&path)?;
    path.push(format!("{version}.json"));
    let component_data = match component_data_result {
        Err(e) => match fs::read(path) {
            Err(_) => Err(e)?,
            Ok(r) => r,
        },
        Ok(r) => {
            fs::write(path, &r)?;
            r.into()
        }
    };
    Ok(serde_json::from_slice(&component_data)?)
}

pub async fn version_exists(path: String, version: String) -> bool {
    let response = reqwest::get(format!("{META}{path}/index.json"))
        .await
        .expect("an error occurred while fetching data from meta");

    let index: Index = serde_json::from_str(
        response
            .text()
            .await
            .expect("error while reading body")
            .as_str(),
    )
    .expect("error while converting to json");
    let mut found: bool = false;
    for item in index {
        if item.version == version {
            found = true;
        }
    }
    found
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
