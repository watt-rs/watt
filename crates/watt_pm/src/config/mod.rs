/// Imports
use crate::errors::PackageError;
use camino::Utf8PathBuf;
use watt_common::bail;
use watt_compile::io;
use serde::{Deserialize, Serialize};
use std::fs;

/// Package type
#[derive(Deserialize, Serialize)]
pub enum PackageType {
    #[serde(rename = "lib")]
    Lib,
    #[serde(rename = "app")]
    App,
}

/// Package config
#[derive(Deserialize, Serialize)]
#[allow(dead_code)]
pub struct PackageConfig {
    pub pkg: PackageType,
    pub name: String,
    pub main: Option<String>,
    pub dependencies: Vec<String>,
}

/// Lints config
#[derive(Deserialize, Serialize)]
#[allow(dead_code)]
pub struct LintsConfig {
    pub disabled: Vec<String>,
}

/// watt.toml
#[derive(Deserialize, Serialize)]
pub struct WattConfig {
    pub pkg: PackageConfig,
    pub lints: LintsConfig,
}

/// Parses config
pub fn parse(path: &Utf8PathBuf, text: String) -> WattConfig {
    match toml::from_str(&text) {
        Ok(cfg) => cfg,
        Err(_) => bail!(PackageError::FailedToParseConfig { path: path.clone() }),
    }
}

/// Locates and reads config file
pub fn locate(path: &Utf8PathBuf) -> Result<String, PackageError> {
    let config_path = path.join("watt.toml");
    match fs::read_to_string(&config_path) {
        Ok(text) => Ok(text),
        Err(_) => Err(PackageError::FailedToFindConfig { path: path.clone() }),
    }
}

/// Locates and parses config
pub fn retrieve_config(path: &Utf8PathBuf) -> WattConfig {
    parse(
        path,
        match locate(path) {
            Ok(text) => text,
            Err(error) => bail!(error),
        },
    )
}

/// Generates config
/// saves into `watt.toml` file in `path`
pub fn generate(path: Utf8PathBuf, name: &str, ty: PackageType, main: Option<String>) {
    match locate(&path) {
        Ok(_) => bail!(PackageError::FailedToGenConfig { path }),
        Err(_) => {
            let config = WattConfig {
                pkg: PackageConfig {
                    pkg: ty,
                    name: name.to_owned(),
                    main,
                    dependencies: vec![],
                },
                lints: LintsConfig { disabled: vec![] },
            };
            let serialized = match toml::to_string(&config) {
                Ok(text) => text,
                Err(_) => bail!(PackageError::FailedToSerializeConfig { path }),
            };
            let config_path = path.join("watt.toml");
            io::write(config_path, serialized);
        }
    }
}
