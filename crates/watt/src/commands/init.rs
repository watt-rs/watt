/// Imports
use crate::errors::CliError;
use camino::Utf8PathBuf;
use watt_common::bail;
use watt_pm::{config::PackageType, generate};
use std::env;

/// Executes command
pub fn execute(ty: Option<String>) {
    // Retrieving current directory
    let cwd = match env::current_dir() {
        Ok(path) => match Utf8PathBuf::try_from(path.clone()) {
            Ok(path) => path,
            Err(_) => bail!(CliError::WrongUtf8Path { path }),
        },
        Err(_) => bail!(CliError::FailedToRetrieveCwd),
    };
    // Getting package type from string
    let pkg_ty = match ty {
        Some(ty) => match ty.as_str() {
            "app" => PackageType::App,
            "lib" => PackageType::Lib,
            _ => bail!(CliError::InvalidPackageType { ty }),
        },
        None => PackageType::App,
    };
    // Generating project
    generate::gen_project(cwd, pkg_ty);
}
