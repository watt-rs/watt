use camino::Utf8PathBuf;
/// Imports
use miette::Diagnostic;
use thiserror::Error;

/// Compile error
#[derive(Debug, Error, Diagnostic)]
pub enum CompileError {
    #[error("path is not a file: {unexpected}.")]
    #[diagnostic(code(parse::unexpected_token), help("module path should be a file."))]
    PathIsNotAFile { unexpected: Utf8PathBuf },
}
