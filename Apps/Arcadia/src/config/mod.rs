pub mod commandline;

use std::env;
use std::io;
use std::path::PathBuf;

pub fn config_root_dir() -> io::Result<PathBuf> {
    let home = env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "home directory not found"))?;

    let mut root = PathBuf::from(home);
    root.push("Arcadia");
    root.push("Configuration");
    Ok(root)
}

pub fn config_file_path(file_name: &str) -> io::Result<PathBuf> {
    if file_name.trim().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "config file name cannot be empty",
        ));
    }

    let mut path = config_root_dir()?;
    path.push(file_name);
    Ok(path)
}
