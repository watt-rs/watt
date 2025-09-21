/// Imports
use serde::Deserialize;
use std::collections::HashMap;

/// Package type
#[derive(Deserialize)]
enum PackageType {
    #[serde(rename = "lib")]
    Lib,
    #[serde(rename = "app")]
    App,
}

/// Package runtime
#[derive(Deserialize)]
enum PackageRuntime {
    #[serde(rename = "bun")]
    Bun,
    #[serde(rename = "deno")]
    Deno,
    #[serde(rename = "common")]
    Common,
}

/// Project config
#[derive(Deserialize)]
struct ProjectConfig {
    pkg: PackageType,
    main: String,
    runtime: PackageRuntime,
}

/// Oil.toml
#[derive(Deserialize)]
struct OilConfig {
    project: ProjectConfig,
    dependencies: HashMap<String, String>,
}

/// Parses config
fn parse(text: String) -> Option<OilConfig> {
    todo!()
}
