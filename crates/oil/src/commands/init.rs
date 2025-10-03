/// Imports
use std::env;
use camino::Utf8PathBuf;
use oil_common::bail;
use oil_pm::config::config;
use crate::errors::CliError;

/// Executes command
pub fn execute() {
    // Retrieving current directory
    let cwd = match env::current_dir() {
        Ok(path) => match Utf8PathBuf::try_from(path.clone()) {
            Ok(path) => path,
            Err(_) => bail!(CliError::WrongUtf8Path { path }),
        },
        Err(_) => bail!(CliError::FailedToRetrieveCwd),
    };
}
