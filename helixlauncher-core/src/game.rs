use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct MinecraftIndexItem {
    version: String,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(transparent)]
struct MinecraftIndexResponse {
    items: Vec<MinecraftIndexItem>,
}

pub async fn get_libraries(version_str: &str, library_path: &Path) -> Option<()> {
    // TODO
    // 1: Fetch meta for version_str
    //    i: Could a cache be implemented?
    // 2: Check if all required libraries are in library_path
    //    i: If yes, return Ok with the paths for the libraries
    // 3: Download required libraries from the paths specified in meta
    None
}

/*pub async fn mc_version_exists(version: String) -> bool {
    let response = reqwest::get("https://meta.helixlauncher.dev/net.minecraft/index.json").await.expect("Meta server not found"); // TODO don't hardcode meta maybe?
    let index: MinecraftIndexResponse = response.json().await.unwrap();
    let mut found: bool = false;
    for x in index.items {
        if x.version == version {
            found = true;
        }
    }
    found
}*/
