use std::path::PathBuf;

pub struct Instance {
    name: String,
    launch: InstanceLaunch,
}

#[derive(Default)]
pub struct InstanceLaunch {
    args: Option<Vec<String>>,
    jvm_args: Option<Vec<String>>,
    prelaunch_command: Option<String>,
    postlaunch_command: Option<String>,
    allocation: Option<RamAllocation>,
    javaagent: Option<PathBuf>,
}

type Mebibytes = u32;

struct RamAllocation {
    min: Mebibytes,
    max: Mebibytes,
}

impl Instance {
    /// Make a new instance.
    ///
    /// ```
    /// let name = "New instance"
    /// let instance = Instance::new(name, InstanceLaunch::default());
    /// ```
    fn new(name: &str, launch: &InstanceLaunch) -> Self {
        // what are the semantics here
        // should an instance be "installed" here
        // before it is allowed to be used?
        // (what would installing an instance entail?)
        // (would it just mean making the .minecraft dir
        //  and the data file?)
        todo!();
    }

    /// Fetch instance from it's path.
    ///
    /// ```
    /// let path = PathBuf::from(r"/home/user/.launcher/instance/minecraft");
    /// let instance = Instance::from(path);
    /// ```
    fn from_path(instance_path: &PathBuf) -> Self {
        todo!();
    }
}
