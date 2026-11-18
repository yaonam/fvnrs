use std::{io, path::PathBuf, sync::OnceLock};

use engine_core::io::root;

type AssetLoader = Box<dyn Fn(&str) -> Option<Vec<u8>> + Send + Sync>;

/// Platform override for reading assets (e.g. Android's AssetManager). Falls back to the filesystem.
static ASSET_LOADER: OnceLock<AssetLoader> = OnceLock::new();

pub fn set_asset_loader(loader: impl Fn(&str) -> Option<Vec<u8>> + Send + Sync + 'static) {
    let _ = ASSET_LOADER.set(Box::new(loader));
}

pub fn assets_dir() -> PathBuf {
    root().join("game/assets")
}

/// Reads an asset by its path relative to the assets dir (forward slashes).
pub fn read_asset(relative: &str) -> io::Result<Vec<u8>> {
    match ASSET_LOADER.get() {
        Some(loader) => loader(relative).ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, format!("asset not found: {relative}"))
        }),
        None => std::fs::read(assets_dir().join(relative)),
    }
}
