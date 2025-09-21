/// Imports
use crate::{
    config::config::{self, OilConfig, PackageConfig},
    dependencies::dependencies,
};
use camino::Utf8PathBuf;
use log::info;
use std::collections::{HashMap, HashSet};

/// Locates and parses config
fn retrieve_config(path: Utf8PathBuf) -> OilConfig {
    config::parse(path.clone(), config::locate(path))
}

/// Resolves dependencies,
/// returns toposorted vector of packages
///
/// * cache -- .cache path
/// * solved -- already solved packages
/// * name -- package name
/// * config -- package config
///
fn packages(
    cache: Utf8PathBuf,
    solved: &mut HashMap<String, String>,
    name: String,
    config: PackageConfig,
) -> &mut HashMap<String, String> {
    info!("Resolving packages that {name} depends on.");
    // Dependencies
    for dependency in config.dependencies {
        // https://github.com/oil-rs/std -> std
        // https://org.gittea.com/repo -> repo
        // ...
        let downloaded = dependencies::download(&dependency, cache.clone());
        let config = retrieve_config(downloaded.0);
        info!("+ Found dependency {} of {name}", &downloaded.1);
        solved.insert(name.clone(), downloaded.1.clone());
        if !solved.contains_key(&name) {
            packages(cache.clone(), solved, downloaded.1, config.pkg);
        }
    }
    solved
}

/// Solves dependencies
fn solve(cache: Utf8PathBuf, name: String, config: PackageConfig) -> HashMap<String, String> {
    return packages(cache, &mut HashMap::new(), name, config).to_owned();
}

/// Runs code
pub fn run(path: Utf8PathBuf, name: String) {
    // Cache path
    let mut cache_path = path.clone();
    cache_path.push(".cache");
    // Config
    let config = retrieve_config(path);
    // Getting toposorted dependencies
    info!("Resolving packages...");
    let resolved = solve(cache_path, name, config.pkg);
    info!("Successfully solved: {:?}", resolved);
}
