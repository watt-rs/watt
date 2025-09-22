/// Imports
use crate::{config::config, dependencies::dependencies};
use camino::Utf8PathBuf;
use log::info;

/// Runs code
pub fn run(path: Utf8PathBuf, name: String) {
    // Cache path
    let mut cache_path = path.clone();
    cache_path.push(".cache");
    // Config
    let config = config::retrieve_config(path);
    // Getting toposorted dependencies
    info!("Resolving packages...");
    let resolved = dependencies::solve(cache_path, name, config.pkg);
    info!("Successfully solved: {:?}", resolved);
}
