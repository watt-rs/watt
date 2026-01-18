/// Imports
use crate::errors::CliError;
use camino::Utf8PathBuf;
use std::env;
use watt_common::bail;
use watt_pm::compile;

/// Executes command
pub fn execute() {
    let cwd = match env::current_dir() {
        Ok(path) => match Utf8PathBuf::try_from(path.clone()) {
            Ok(path) => path,
            Err(_) => bail!(CliError::WrongUtf8Path { path }),
        },
        Err(_) => bail!(CliError::FailedToRetrieveCwd),
    };

    compile::analyze(cwd);
}
