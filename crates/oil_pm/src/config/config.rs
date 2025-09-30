/// Imports
use crate::errors::PackageError;
use camino::Utf8PathBuf;
use oil_common::bail;
use serde::Deserialize;
use std::fs;

/// Package type
#[derive(Deserialize)]
pub enum PackageType {
    #[serde(rename = "lib")]
    Lib,
    #[serde(rename = "app")]
    App,
}

/// Package config
#[derive(Deserialize)]
#[allow(dead_code)]
pub struct PackageConfig {
    pub pkg: PackageType,
    pub name: String,
    pub main: Option<String>,
    pub dependencies: Vec<String>,
}

/// Oil.toml
#[derive(Deserialize)]
pub struct OilConfig {
    pub pkg: PackageConfig,
}

/// Parses config
pub fn parse(path: Utf8PathBuf, text: String) -> OilConfig {
    match toml::from_str(&text) {
        Ok(cfg) => cfg,
        Err(_) => bail!(PackageError::FailedToParseConfig { path }),
    }
}

/// Locates and reads config file
pub fn locate(path: Utf8PathBuf) -> String {
    let mut config_path = path.clone();
    config_path.push("oil.toml");
    match fs::read_to_string(&config_path) {
        Ok(text) => text,
        Err(_) => bail!(PackageError::FailedToFindConfig { path }),
    }
}

/// Locates and parses config
pub fn retrieve_config(path: Utf8PathBuf) -> OilConfig {
    parse(path.clone(), locate(path))
}
