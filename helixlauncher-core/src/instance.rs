use std::{
    fs::{self, File},
    io::BufReader,
    path::Path,
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InstanceManagerError {
    #[error(transparent)]
    ParseError(#[from] serde_json::Error),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error("Path given is not an instance")]
    NotAnInstance,
}

pub enum Modloader {
    Quilt,
    Fabric,
    Forge,
    Vanilla,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Instance {
    pub name: String,
    pub components: Vec<Component>,
    pub launch: InstanceLaunch,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct InstanceLaunch {
    pub args: Vec<String>,
    pub jvm_args: Vec<String>,
    pub prelaunch_command: Option<String>,
    pub postlaunch_command: Option<String>,
    pub allocation: Option<RamAllocation>,
    // javaagent: Option<PathBuf>,
}

type Mebibytes = u32;

#[derive(Serialize, Deserialize, Debug)]
pub struct RamAllocation {
    min: Mebibytes,
    max: Mebibytes,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Component {
    pub id: String,
    pub version: String,
}

const INSTX_CONFIG_NAME: &str = "instance.helix.json";
const SUBDIR_CONFIG_NAME: &str = "directory.helix.json";

impl Instance {
    /// Make a new instance.
    ///
    /// ```
    /// let name = "New instance";
    /// let instances_dir = PathBuf::from(r"/home/user/.launcher/instance/")
    /// let instance = Instance::new(name, InstanceLaunch::default());
    /// ```
    pub fn new(
        name: String,
        mc_version: String,
        launch: InstanceLaunch,
        instances_dir: &Path,
        modloader: Modloader,
        modloader_version: Option<String>,
    ) -> Result<Self, InstanceManagerError> {
        let modloader_component_string: String = match modloader {
            Modloader::Fabric => String::from("net.fabricmc.fabric-loader"),
            Modloader::Quilt => String::from("org.quiltmc.quilt-loader"),
            Modloader::Forge => String::from("net.minecraftforge.forge"),
            Modloader::Vanilla => String::from(""),
        };

        let mut components = vec![Component { id: String::from("net.minecraft"), version: mc_version}];
        /*match modloader_component_string {
            String::from("") => {
                
            }
            _ => {
                vec![Component { id: String::from("net.minecraft"), version: mc_version, Component {
                    id: modloader_component_string,
                    version: modloader_version.unwrap()
                }}]
                
            }
        };*/
        if modloader_component_string != String::from("") {
            components.append(&mut vec![Component { id: modloader_component_string, version: modloader_version.unwrap()}]);
        }
        let instance = Self {
            name,
            components,
            launch,
        };

        // make instance folder & skeleton (try to avoid collisions)
        let instance_dir = instances_dir.join(&instance.name);
        if instance_dir.try_exists()? {
            todo!("Resolve folder collision (1)");
        }

        // make the .minecraft dir & instance dir in one line
        fs::create_dir_all(instance_dir.join(".minecraft"))?;

        // create instance config
        let instance_json = File::create(instance_dir.join(INSTX_CONFIG_NAME))?;
        serde_json::to_writer_pretty(instance_json, &instance)?;

        Ok(instance)
    }

    /// Fetch instance from its path.
    ///
    /// ```
    /// let path = PathBuf::from(r"/home/user/.launcher/instance/minecraft");
    /// let instance = Instance::from(path);
    /// ```
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, InstanceManagerError> {
        // search for top-level config file, return error if not there
        match fs::read_dir(path)?
            .filter_map(|x| x.ok())
            .find(|file| file.file_name() == INSTX_CONFIG_NAME)
        {
            Some(config) => {
                let reader = BufReader::new(File::open(config.path())?);
                Ok(serde_json::from_reader(reader)?)
            }
            None => Err(InstanceManagerError::NotAnInstance),
        }
    }

    pub fn list_instances<P: AsRef<Path>>(
        instances_dir: P,
    ) -> Result<Vec<Self>, InstanceManagerError> {
        fs::read_dir(instances_dir)?
            .map(|i| Self::from_path(i?.path()))
            .collect()
    }
}
