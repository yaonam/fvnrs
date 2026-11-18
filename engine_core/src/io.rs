use std::{
    fs::File,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use anyhow::Result;
use serde::{Serialize, de::DeserializeOwned};

static ROOT_PATH: OnceLock<PathBuf> = OnceLock::new();

pub fn init_root_path(path: PathBuf) {
    let _ = ROOT_PATH.set(path);
}

pub fn read_file<T: DeserializeOwned, P: AsRef<Path>>(path: P) -> Result<T> {
    let file = File::open(path)?;
    let t: T = serde_json::from_reader(file)?;
    Ok(t)
}

pub fn write_file<T: Serialize, P: AsRef<Path>>(path: P, t: T) -> Result<()> {
    // Create directory if it doesn't exist
    if let Some(parent) = path.as_ref().parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let file = File::create(path)?;
    serde_json::to_writer(file, &t)?;
    Ok(())
}

/// Repo root when launched with `cargo run` from there, or the install directory when the packaged exe is started from its own folder.
pub fn root() -> PathBuf {
    ROOT_PATH.get().cloned().expect("root path initialized")
}

pub fn persistent_path() -> PathBuf {
    root().join("persistent.json")
}

pub fn saves_dir() -> PathBuf {
    root().join("saves")
}

pub fn remove_file<P: AsRef<Path>>(path: P) -> Result<()> {
    std::fs::remove_file(path)?;
    Ok(())
}

/// Returns the files in the directory, sorted by name.
pub fn list_files(path: impl AsRef<Path>, extension: impl AsRef<str>) -> Result<Vec<String>> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file()
            && path
                .extension()
                .is_some_and(|ext| ext == extension.as_ref())
        {
            files.push(path.to_string_lossy().to_string());
        }
    }
    files.sort();
    Ok(files)
}
