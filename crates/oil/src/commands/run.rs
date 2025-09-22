/// Imports
use crate::{config::config, dependencies::dependencies};
use camino::Utf8PathBuf;
use log::info;
use oil_compile::project::ProjectCompiler;

/// Runs code
pub fn run(path: Utf8PathBuf, name: String) {
    // Cache path
    let mut cache_path = path.clone();
    cache_path.push(".cache");
    // Config
    let config = config::retrieve_config(path.clone());
    // Getting toposorted packages
    info!("Resolving packages...");
    let resolved = dependencies::solve(cache_path.clone(), name.clone(), config.pkg);
    info!("Successfully solved: {:?}", resolved);
    // Packages paths
    let packages_paths = {
        resolved.iter().map(|pkg| {
            // If it's our package
            if &name == pkg {
                path.clone()
            } else {
                let mut pkg_path = cache_path.clone();
                pkg_path.push(pkg);
                pkg_path
            }
        })
    }
    .collect();
    // Target path
    let target_path = {
        let mut target_path = Utf8PathBuf::new();
        target_path.push(&path);
        target_path.push("target");
        target_path
    };
    // Compiling
    let mut project_compiler = ProjectCompiler::new(packages_paths, target_path);
    project_compiler.compile();
}
