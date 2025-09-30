/// Imports
use miette::Diagnostic;
use thiserror::Error;
use std::path::PathBuf;

/// Cli error
#[derive(Debug, Error, Diagnostic)]
pub enum CliError {
    #[error("failed to retrieve current working directory.")]
    #[diagnostic(
        code(pkg::failed_to_retrieve_cwd),
        help("check existence of current working directory.")
    )]
    FailedToRetrieveCwd,
    #[error("failed to convert path {path} to utf8 path.")]
    #[diagnostic(code(pkg::wrong_utf8_path))]
    WrongUtf8Path { path: PathBuf },
}
