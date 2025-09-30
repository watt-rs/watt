/// Imports
use crate::errors::CliError;
use camino::Utf8PathBuf;
use oil_common::bail;
use oil_pm::{compile, runtime::JsRuntime};
use std::env;

/// Runs code
fn run(path: Utf8PathBuf, runtime: JsRuntime) {
    // Running code
    compile::run(path, runtime);
}

/// Executes command
pub fn execute(runtime: JsRuntime) {
    // Retrieving current directory
    let cwd = match env::current_dir() {
        Ok(path) => match Utf8PathBuf::try_from(path.clone()) {
            Ok(path) => path,
            Err(_) => bail!(CliError::WrongUtf8Path { path }),
        },
        Err(_) => bail!(CliError::FailedToRetrieveCwd),
    };
    // Running code
    run(cwd, runtime)
}
