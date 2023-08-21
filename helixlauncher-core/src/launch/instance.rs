use std::{
    fmt::Display,
    fs::{self, File},
    io::{self, BufReader},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    config::Config,
    meta::{ComponentMetaRetrievalError, HelixLauncherMeta},
};

#[derive(Error, Debug)]
pub enum InstanceManagerError {
    #[error(transparent)]
    ParseError(#[from] serde_json::Error),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error("Path given is not an instance")]
    NotAnInstance,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Modloader {
    Quilt,
    Fabric,
    Forge,
    Vanilla,
}

impl Display for Modloader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct Instance {
    pub path: PathBuf,
    pub config: InstanceConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InstanceConfig {
    pub name: String,
    pub components: Vec<Component>,
    pub launch: InstanceLaunchConfig,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct InstanceLaunchConfig {
    // Options are taken from launcher settings if absent
    pub args: Option<Vec<String>>,
    pub jvm_args: Option<Vec<String>>,
    pub prelaunch_command: Option<String>,
    pub postlaunch_command: Option<String>,
    pub allocation: Option<RamAllocation>,
    pub javapath: Option<String>, // FIXME: this is less than ideal, needs more settings and better
                                  // defaults
}

type Mebibytes = u32;

#[derive(Serialize, Deserialize, Debug)]
pub struct RamAllocation {
    pub min: Mebibytes,
    pub max: Mebibytes,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Component {
    pub id: String,
    pub version: String,
}

impl Component {
    pub async fn into_meta(
        &self,
        config: &Config,
    ) -> Result<helixlauncher_meta::component::Component, ComponentMetaRetrievalError> {
        HelixLauncherMeta::new(config)
            .get_component_meta(&self.id, &self.version)
            .await
    }
}

const INSTANCE_CONFIG_NAME: &str = "instance.helix.json";
const _SUBDIR_CONFIG_NAME: &str = "directory.helix.json";

impl Instance {
    /// Make a new instance.
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use helixlauncher_core::launch::instance::{Instance, InstanceLaunchConfig, Modloader};
    /// # use std::path::PathBuf;
    /// # let data_tempdir = tempfile::tempdir()?;
    /// # let data_dir = data_tempdir.path();
    /// let instances_dir = data_dir.join("instances");
    /// let instance = Instance::new(String::from("New instance"), String::from("1.20"), InstanceLaunchConfig::default(), &instances_dir, Modloader::Vanilla, None);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(
        name: String,
        mc_version: String,
        launch: InstanceLaunchConfig,
        instances_dir: &Path,
        modloader: Modloader,
        modloader_version: Option<String>,
    ) -> Result<Self, InstanceManagerError> {
        // TODO: maybe make this more generic? and let the caller specify the components
        let modloader_component_id = match modloader {
            Modloader::Fabric => Some("net.fabricmc.fabric-loader"),
            Modloader::Quilt => Some("org.quiltmc.quilt-loader"),
            Modloader::Forge => Some("net.minecraftforge.forge"),
            Modloader::Vanilla => None,
        };

        let mut components = vec![Component {
            id: String::from("net.minecraft"),
            version: mc_version,
        }];

        if let Some(modloader_component_id) = modloader_component_id {
            components.insert(
                0,
                Component {
                    id: String::from(modloader_component_id),
                    version: modloader_version.unwrap(),
                },
            );
        }

        // make instance folder & skeleton (try to avoid collisions)
        let instance_dir = instances_dir.join(&name);
        if instance_dir.try_exists()? {
            todo!("Resolve folder collision (1)");
        }

        // make the .minecraft dir & instance dir in one line
        fs::create_dir_all(instance_dir.join(".minecraft"))?;

        let instance_json_path = instance_dir.join(INSTANCE_CONFIG_NAME);

        let instance = Self {
            path: instance_dir,
            config: InstanceConfig {
                name,
                components,
                launch,
            },
        };

        // create instance config
        let instance_json = File::create(instance_json_path)?;
        serde_json::to_writer_pretty(instance_json, &instance.config)?;

        Ok(instance)
    }

    /// Fetch instance from its path.
    ///
    /// ```
    /// # use helixlauncher_core::launch::instance::Instance;
    /// let instance = Instance::from_path(r"/home/user/.launcher/instance/minecraft");
    /// ```
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, InstanceManagerError> {
        let path = PathBuf::from(path.as_ref());
        // search for top-level config file, return error if not there
        Ok(Instance {
            config: serde_json::from_reader(BufReader::new(match File::open(
                path.join(INSTANCE_CONFIG_NAME),
            ) {
                Err(e) if e.kind() == io::ErrorKind::NotFound => {
                    Err(InstanceManagerError::NotAnInstance)
                }
                r => r.map_err(InstanceManagerError::from),
            }?))?,
            path,
        })
    }

    pub fn list_instances<P: AsRef<Path>>(
        instances_dir: P,
    ) -> Result<Vec<Self>, InstanceManagerError> {
        fs::read_dir(instances_dir)?
            .map(|i| Self::from_path(i?.path()))
            .collect()
    }

    pub fn get_game_dir(&self) -> PathBuf {
        self.path.join(".minecraft")
    }

    pub fn get_component_version(&self, id: &str) -> Option<&str> {
        self.config
            .components
            .iter()
            .filter(|component| component.id == id)
            .map(|component| &*component.version)
            .next()
    }
}
