use std::path::Path;

use helixlauncher_meta::index::Index;
use serde::{Deserialize, Serialize};

pub async fn get_libraries(version_str: &str, library_path: &Path) -> Option<()> {
    // TODO
    // 1: Fetch meta for version_str
    //    i: Could a cache be implemented?
    // 2: Check if all required libraries are in library_path
    //    i: If yes, return Ok with the paths for the libraries
    // 3: Download required libraries from the paths specified in meta
    None
}

pub async fn version_exists(path: String, version: String) -> bool {
    let response = reqwest::get(format!(
        "https://meta.helixlauncher.dev/{}/index.json",
        path.as_str()
    ))
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
