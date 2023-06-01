pub mod curseforge;
pub mod modrinth;

use serde::{Serialize, Deserialize};

use crate::launch::instance::{Modloader, Instance};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ApiKind {
    Curseforge,
    Modrinth
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum AddonKind {
    Mod,
    /// Legacy Forge stuff
    CoreMod,
    Resource,
    /// Old versions use `texturepacks`, this is easier than supplying a version number
    Texture,
    Shader
}

#[derive(Serialize, Deserialize)]
struct AddonLocation {
    /// Determines what api we should use to download the file
    pub api: ApiKind,
    /// The version currently downloaded
    pub version_id: String
}

#[derive(Serialize, Deserialize)]
struct Addon {
    /// The file we check against when verifying/updating
    pub file_name: String,
    /// This determines the folder it should go in
    pub kind: AddonKind,
    /// The loaders this addon works on (empty or [Vanilla] for all)
    pub loaders: Vec<Modloader>,
    /// The versions allowed by this addon
    pub game_versions: Vec<String>,
    /// File hash to check against
    pub hash: String,
    /// Location data for the origin
    pub location: AddonLocation
}

impl AddonKind {
    pub fn get_folder(&self) -> String {
        match self {
            AddonKind::Mod => "mods".into(),
            AddonKind::CoreMod => "coremods".into(),
            AddonKind::Resource => "resourcepacks".into(),
            AddonKind::Texture => "texturepacks".into(),
            AddonKind::Shader => "shaderpacks".into(),
        }
    }
}

impl AddonLocation {
    pub fn curseforge(version_id: String) -> Self {
        Self {
            api: ApiKind::Curseforge,
            version_id
        }
    }

    pub fn modrinth(version_id: String) -> Self {
        Self {
            api: ApiKind::Modrinth,
            version_id
        }
    }

    pub fn download(&self, instance: &Instance) {
        match self.api {
            ApiKind::Curseforge => self.download_curseforge(instance),
            ApiKind::Modrinth => self.download_modrinth(instance)
        }
    }
}

impl Addon {
    pub fn verify(&self, instance: &Instance) -> bool {
        todo!()
    }

    pub fn update(&mut self, instance: &Instance) {
        todo!()
    }

    pub fn delete(&self, instance: &Instance) {
        todo!()
    }
}
