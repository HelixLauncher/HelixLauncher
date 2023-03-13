use std::{
    fs, io,
    path::{self, Path},
};

pub fn copy_file(from: &Path, to: &Path) -> io::Result<()> {
    // TODO: investigate reflinks
    fs::copy(from, to)?;
    Ok(())
}

const ILLEGAL_FILENAMES: &[&str] = &[
    "aux", "com0", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8", "com9", "con",
    "lpt0", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9", "nul", "prn",
];

// This is not an exhaustive check on being a _valid_ path, but should be one on being a
// _dangerous_ path.
pub fn check_path(path: &str) -> bool {
    for component in Path::new(path).components() {
        if let path::Component::Normal(component) = component {
            let name = component.to_str().unwrap();
            if name.contains(|c| matches!(c, '\0'..='\x1f' | '$' | ':')) {
                return false;
            }
            let prefix = name.split_once('.').map_or(name, |t| t.0);
            if ILLEGAL_FILENAMES
                .binary_search(&prefix.to_ascii_lowercase().as_ref())
                .is_ok()
            {
                return false;
            }
        } else {
            return false;
        }
    }
    true
}
