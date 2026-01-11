/// Imports
use crate::errors::CliError;
use camino::Utf8PathBuf;
use std::env;
use watt_common::bail;
use watt_pm::{config::PackageType, generate};

/// Executes command
pub fn execute(pkg_ty: Option<PackageType>) {
    // Retrieving current directory
    let cwd = match env::current_dir() {
        Ok(path) => match Utf8PathBuf::try_from(path.clone()) {
            Ok(path) => path,
            Err(_) => bail!(CliError::WrongUtf8Path { path }),
        },
        Err(_) => bail!(CliError::FailedToRetrieveCwd),
    };

    // Getting package type from string
    let pkg_ty = pkg_ty.unwrap_or(PackageType::App);

    // Generating project
    generate::gen_project(cwd, pkg_ty);
}
