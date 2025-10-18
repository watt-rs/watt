use std::path::PathBuf;

/// Imports
use camino::Utf8PathBuf;
use miette::Diagnostic;
use thiserror::Error;

/// Io error
#[derive(Debug, Error, Diagnostic)]
pub enum IoError {
    #[error("failed to read watt file: {path}.")]
    #[diagnostic(
        code(io::failed_to_read),
        help("please, file an issue on github."),
        url("https://github.com/wattlanguage/watt")
    )]
    FailedToRead { path: Utf8PathBuf },
    #[error("failed to write in file: {path}.")]
    #[diagnostic(code(io::failed_to_write))]
    FailedToWrite { path: Utf8PathBuf },
    #[error("failed to make directory: {path}.")]
    #[diagnostic(code(io::failed_to_mkdir))]
    FailedToMkdir { path: Utf8PathBuf },
    #[error("failed to make directory tree: {path}.")]
    #[diagnostic(code(io::failed_to_mkdir_all))]
    FailedToMkdirAll { path: Utf8PathBuf },
    #[error("entry error inside children of: {path}.")]
    #[diagnostic(
        code(io::failed_to_read),
        help("please, file an issue on github."),
        url("https://github.com/wattlanguage/watt")
    )]
    EntryError { path: Utf8PathBuf },
    #[error("failed to convert PathBuf of: {path} to Utf8PathBuf.")]
    #[diagnostic(
        code(io::failed_to_convert_path_buf_to_utf8_path_buf),
        help("please, check file name is valid utf-8 string.")
    )]
    FailedToConvertPathBuf { path: PathBuf },
    #[error("failed to strip prefix {root} of {path}.")]
    #[diagnostic(
        code(io::failed_to_convert_path_buf_to_utf8_path_buf),
        help("please, file an issue on github."),
        url("https://github.com/wattlanguage/watt")
    )]
    FailedToStripPrefix {
        path: Utf8PathBuf,
        root: Utf8PathBuf,
    },
}
