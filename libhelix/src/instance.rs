use std::{
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Instance {
    name: String,
    launch: InstanceLaunch,
}

#[derive(Default, Serialize, Deserialize)]
pub struct InstanceLaunch {
    args: Vec<String>,
    jvm_args: Vec<String>,
    prelaunch_command: Option<String>,
    postlaunch_command: Option<String>,
    allocation: Option<RamAllocation>,
    javaagent: Option<PathBuf>,
}

type Mebibytes = u32;

#[derive(Serialize, Deserialize)]
struct RamAllocation {
    min: Mebibytes,
    max: Mebibytes,
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
    fn new(name: String, launch: InstanceLaunch, instances_dir: &Path) -> Self {
        let instance = Self { name, launch };

        // make instance folder & skeleton (try to avoid collisions)
        let mut instance_dir = instances_dir.join(&instance.name);
        if instances_dir.try_exists().unwrap() {
            todo!("Resolve folder collision (1)");
        }

        instance_dir.push(".minecraft");
        fs::create_dir_all(&instance_dir).unwrap();

        // create instance config
        instance_dir.push(INSTX_CONFIG_NAME);
        let instance_json = File::create(&instance_dir).unwrap();
        serde_json::to_writer_pretty(instance_json, &instance).unwrap();

        instance
    }

    /// Fetch instance from its path.
    ///
    /// ```
    /// let path = PathBuf::from(r"/home/user/.launcher/instance/minecraft");
    /// let instance = Instance::from(path);
    /// ```
    fn from_path<P: AsRef<Path>>(path: P) -> Self {
        if !InstanceFolderSearchItems::is_instance(&path) {
            panic!("put a real option/result here!");
        }
        let instance_json = path.as_ref().join(INSTX_CONFIG_NAME);
        read_json_file(&instance_json)
    }

    fn list_instances<P: AsRef<Path>>(instances_dir: P) -> Vec<Self> {
        fs::read_dir(instances_dir)
            .unwrap()
            .map(|i| Self::from_path(i.unwrap().path()))
            .collect()
    }
}

fn read_json_file<T: DeserializeOwned>(path: &Path) -> T {
    let file = BufReader::new(File::open(path).unwrap());
    serde_json::from_reader(file).unwrap()
}

#[derive(Serialize, Deserialize)]
pub struct InstanceDirectory {
    name: String,
    children: Vec<String>,
    relative_path: PathBuf,
}
impl InstanceDirectory {
    // silly logic:
    // there is always one base instancedir, of course.
    // to add a directory you have to call a method on
    // the parent directory.

    pub fn base(instances_dir: PathBuf) -> Self {
        let directory_json = instances_dir.join(SUBDIR_CONFIG_NAME);

        // case: there is no directory.helix.json
        //     write default, save, return

        if instances_dir.try_exists().unwrap() {
            read_json_file(&directory_json)
        } else {
            // write defaults
            let inst_dir = Self {
                name: String::from("Base Directory"),
                children: vec![],
                relative_path: PathBuf::from("."),
            };
            let directory_json = File::create(directory_json).unwrap();
            serde_json::to_writer_pretty(directory_json, &inst_dir).unwrap();

            inst_dir
        }
    }
    pub fn new(name: String, parent: &mut Self, instances_dir: PathBuf) -> Self {
        let inst_dir = Self {
            name: name.clone(),
            children: vec![],
            relative_path: parent.relative_path.join(name),
        };

        // make this path absolute
        let instance_dir = instances_dir.join(inst_dir.relative_path.clone());

        fs::create_dir_all(&instance_dir).unwrap();

        // write json
        let directory_json = instance_dir.join(SUBDIR_CONFIG_NAME);
        let directory_json = File::create(directory_json).unwrap();
        serde_json::to_writer_pretty(directory_json, &inst_dir).unwrap();

        // edit parent's config to add child
        parent.children.push(inst_dir.name.clone());
        let parent_dir = instances_dir.join(parent.relative_path.clone());
        let directory_json = parent_dir.join(SUBDIR_CONFIG_NAME);
        let directory_json = File::create(directory_json).unwrap();
        serde_json::to_writer_pretty(directory_json, &parent).unwrap();

        inst_dir
    }
}
enum InstanceFolderSearchItems {
    InstanceDir,
    DirectoryDir,
    UnknownDir,
}
impl InstanceFolderSearchItems {
    fn identify_item<P: AsRef<Path>>(path: P) -> Self {
        // weird edge case here: folder could have both files in. if so, whoops!
        fs::read_dir(path)
            .unwrap()
            .find_map(|file| {
                let file_name = file.unwrap().file_name();
                if file_name == INSTX_CONFIG_NAME {
                    Some(InstanceFolderSearchItems::InstanceDir)
                } else if file_name == SUBDIR_CONFIG_NAME {
                    Some(InstanceFolderSearchItems::DirectoryDir)
                } else {
                    None
                }
            })
            .unwrap_or(InstanceFolderSearchItems::UnknownDir)
    }
    fn is_instance<P: AsRef<Path>>(path: P) -> bool {
        // weird edge case here: folder could have both files in. if so, whoops!
        fs::read_dir(path)
            .unwrap()
            .any(|file| file.unwrap().file_name() == INSTX_CONFIG_NAME)
    }
}
