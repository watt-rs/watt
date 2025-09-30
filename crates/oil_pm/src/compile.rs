/// Imports
use crate::{config::config, dependencies::dependencies, errors::PackageError, runtime::JsRuntime};
use camino::{Utf8Path, Utf8PathBuf};
use console::style;
use log::info;
use oil_common::bail;
use oil_compile::{io::io, project::ProjectCompiler};
use std::process::Command;

/// Runs using runtime
fn run_by_rt(index: Utf8PathBuf, rt: JsRuntime) {
    println!(
        "{} Preparing for {rt:?} runtime...",
        style("[📌]").bold().red()
    );
    match rt {
        JsRuntime::Deno => {
            // `deno init $path`
            match Command::new("deno").args(&["run", index.as_str()]).status() {
                Err(error) => bail!(PackageError::FailedToRunProject {
                    rt,
                    error: error.to_string()
                }),
                _ => {}
            }
        }
        JsRuntime::Node => {
            // `npm init -y` in target path
            match Command::new("node").args(&[index.as_str()]).status() {
                Err(error) => bail!(PackageError::FailedToRunProject {
                    rt,
                    error: error.to_string()
                }),
                _ => {}
            }
        }
        JsRuntime::Bun => {
            // `bun init -y` in target path
            match Command::new("bun").arg(index.as_str()).status() {
                Err(error) => bail!(PackageError::FailedToRunProject {
                    rt,
                    error: error.to_string()
                }),
                _ => {}
            }
        }
        JsRuntime::Common => {}
    }
}

/// Compiles project to js
pub fn compile(path: Utf8PathBuf, name: String, rt: JsRuntime) {
    // Cache path
    let mut cache_path = path.clone();
    cache_path.push(".cache");
    // Config
    let config = config::retrieve_config(path.clone());
    // Getting toposorted packages
    println!("{} Resolving packages...", style("[🔍]").bold().cyan());
    let resolved = dependencies::solve(cache_path.clone(), name.clone(), &config.pkg);
    println!("{} Packages resolved.", style("[✓]").bold().cyan());
    info!("Resolved packages: {resolved:?}");
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
    println!("{} Compiling...", style("[🚚]").bold().yellow());
    let mut project_compiler = ProjectCompiler::new(packages_paths, &target_path);
    let completed_packages = project_compiler.compile();
    // Retrieving main package from completed packages
    let main_package = match completed_packages
        .into_iter()
        .find(|package| package.path == path)
    {
        Some(package) => package,
        None => bail!(PackageError::NoMainPackageFound { path: path }),
    };
    // Retrieving main module name from config
    let main_module_name = match &config.pkg.main {
        Some(m) => m,
        None => bail!(PackageError::NoMainModuleFoundSpecified { path }),
    };
    // Retrieving main module with $main_module_name
    // from the main package, checking for module existence
    let main_module = match main_package
        .modules
        .into_iter()
        .find(|module| &module.name == main_module_name)
    {
        Some(m) => m,
        None => bail!(PackageError::NoMainModuleFound {
            module: main_module_name.clone()
        }),
    };
    // Generating `index.js`
    let mut index_path = Utf8PathBuf::from(target_path);
    index_path.push(Utf8Path::new("index.js"));
    let mut index_data = String::new();
    index_data.push_str("import {main} from \"");
    index_data.push_str(&format!("./{}.js", main_module.name));
    index_data.push('"');
    index_data.push('\n');
    index_data.push_str("main();");
    io::write(index_path.clone(), index_data);
    // Checking for main function
    if let None = main_module.analyzed.fields.get("main") {
        bail!(PackageError::NoMainFnFound {
            module: main_module_name.clone()
        });
    }
    // Done
    println!("{} Done.", style("[✓]").bold().yellow());
    // Running
    run_by_rt(index_path, rt);
}
