/// Imports
use crate::{
    config::{self, WattConfig},
    dependencies::{self, Package},
    errors::PackageError,
    runtime::JsRuntime, url::path_to_pkg_name,
};
use camino::{Utf8Path, Utf8PathBuf};
use console::style;
use std::process::Command;
use tracing::info;
use watt_common::{
    bail,
    package::{DraftPackage, DraftPackageLints},
    skip,
};
use watt_compile::{io, package::CompletedPackage, project::ProjectCompiler};

/// Runs using runtime
fn run_by_rt(index: Utf8PathBuf, rt: JsRuntime) {
    println!(
        "{} Preparing for {rt:?} runtime...",
        style("[📌]").bold().red()
    );
    match rt {
        JsRuntime::Deno => {
            // `deno init $path`
            if let Err(error) = Command::new("deno").args(["run", index.as_str()]).status() {
                bail!(PackageError::FailedToRunProject {
                    rt,
                    error: error.to_string()
                })
            }
        }
        JsRuntime::Node => {
            // `npm init -y` in target path
            if let Err(error) = Command::new("node").args([index.as_str()]).status() {
                bail!(PackageError::FailedToRunProject {
                    rt,
                    error: error.to_string()
                })
            }
        }
        JsRuntime::Bun => {
            // `bun init -y` in target path
            if let Err(error) = Command::new("bun").arg(index.as_str()).status() {
                bail!(PackageError::FailedToRunProject {
                    rt,
                    error: error.to_string()
                })
            }
        }
        JsRuntime::Common => skip!(),
    }
}

/// Writes `index.js`
/// returns path to it
fn write_index(
    completed_packages: &[CompletedPackage],
    project_path: Utf8PathBuf,
    target_path: &Utf8PathBuf,
    config: &WattConfig,
) -> Utf8PathBuf {
    // Retrieving main package from completed packages
    let main_package = match completed_packages
        .iter()
        .find(|package| package.path == project_path)
    {
        Some(package) => package,
        None => bail!(PackageError::NoMainPackageFound { path: project_path }),
    };

    // Retrieving main module name from config
    let main_module_name = match &config.pkg.main {
        Some(m) => m.clone(),
        None => bail!(PackageError::NoMainModuleFoundSpecified { path: project_path }),
    };

    // Retrieving main module with $main_module_name
    // from the main package, checking for module existence
    let main_module = match main_package
        .modules
        .iter()
        .find(|module| module.name == main_module_name)
    {
        Some(m) => m,
        None => bail!(PackageError::NoMainModuleFound {
            module: main_module_name.clone()
        }),
    };

    // Checking for main function
    if !main_module.analyzed.fields.contains_key("main") {
        bail!(PackageError::NoMainFnFound {
            module: main_module_name.clone()
        });
    }

    // Generating `index.js`
    let mut index_path = Utf8PathBuf::from(target_path);
    index_path.push(Utf8Path::new("index.js"));
    io::write(
        index_path.clone(),
        &watt_gen::gen_index(main_module_name)
            .to_file_string()
            .unwrap(),
    );

    index_path
}


/// Compiles project to js
/// returns path to `index.js`
pub fn compile(path: Utf8PathBuf) -> Utf8PathBuf {
    // Cache path
    let mut cache_path = path.clone();
    cache_path.push(".cache");
    // Config
    let config = config::retrieve_config(&path);
    // Retrieving project name
    let name = path_to_pkg_name(&path);
    info!("Crawled project name {name} from {path}.");
    // Getting toposorted packages
    println!("{} Resolving packages...", style("[🔍]").bold().cyan());
    let resolved = dependencies::solve(
        cache_path.clone(),
        Package::Local(name, path.clone()),
        &config.pkg,
    );
    println!("{} Packages resolved.", style("[✓]").bold().cyan());
    info!("Resolved packages: {resolved:?}");
    // Packages paths
    let packages = {
        resolved.iter().map(|pkg| {
            // Package config
            let config = config::retrieve_config(pkg.path());
            // Generating draft package
            DraftPackage {
                path: pkg.path().clone(),
                lints: DraftPackageLints {
                    disabled: config.lints.disabled,
                },
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
    let mut project_compiler = ProjectCompiler::new(packages, &target_path);
    let completed_packages = project_compiler.compile();
    // Writing `index.js`
    let index_path = write_index(&completed_packages, path, &target_path, &config);
    // Done
    println!("{} Done.", style("[✓]").bold().yellow());
    index_path
}

/// Runs project
pub fn run(path: Utf8PathBuf, rt: JsRuntime) {
    // Compiling project
    let index_path = compile(path);
    // Running it
    run_by_rt(index_path, rt);
}
