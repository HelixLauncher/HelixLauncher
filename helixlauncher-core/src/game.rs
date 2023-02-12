use std::{collections::HashMap, error::Error};

use helixlauncher_meta::{
    component::{self, Component, ConditionalClasspathEntry, Hash, Platform},
    index::Index,
    util::{GradleSpecifier, CURRENT_ARCH, CURRENT_OS},
};
use indexmap::IndexMap;

use crate::instance;

const META: &str = "https://meta.helixlauncher.dev/";

#[derive(Debug, Default)]
pub struct MergedComponents {
    pub classpath: Vec<GradleSpecifier>,
    pub natives: Vec<Native>,
    pub artifacts: HashMap<GradleSpecifier, Artifact>,
    pub traits: Vec<component::Trait>,
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

// TODO: proper error (and progress?) handling
// TODO: this doesn't handle stuff like Rosetta or running x86 Java on x86_64 at all
pub async fn merge_components(
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
        let component = fetch_component(&component.id, &component.version).await?;
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
        for jarmod in component.jarmods {
            let unversioned_name = GradleSpecifier {
                version: String::from(""),
                ..jarmod.clone()
            };
            jarmods.entry(unversioned_name).or_insert(jarmod);
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
    })
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

async fn fetch_component(id: &str, version: &str) -> Result<Component, Box<dyn Error>> {
    // TODO: caching
    Ok(reqwest::get(format!("{META}{id}/{version}.json"))
        .await?
        .json()
        .await?)
}

pub async fn version_exists(path: String, version: String) -> bool {
    let response = reqwest::get(format!("{META}{path}/index.json",))
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
