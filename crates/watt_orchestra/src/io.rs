/// Imports
use camino::Utf8PathBuf;
use miette::Diagnostic;
use std::{fs, path::PathBuf};
use thiserror::Error;
use walkdir::WalkDir;
use watt_macros::bail;

/// IO error
#[derive(Debug, Error, Diagnostic)]
pub enum IoError {
    #[error("failed to read watt file: {path}.")]
    #[diagnostic(code(io::failed_to_read))]
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
        code(io::entry_error),
        help("please, file an issue on github."),
        url("https://github.com/watt-rs/watt")
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
        url("https://github.com/watt-rs/watt")
    )]
    FailedToStripPrefix {
        path: Utf8PathBuf,
        root: Utf8PathBuf,
    },
}

/// Returns module name by root path and file path
pub fn module_name(root: &Utf8PathBuf, path: &Utf8PathBuf) -> String {
    // Getting module local path
    let mut module_path = match path.strip_prefix(root) {
        Ok(ok) => ok.to_path_buf(),
        Err(_) => bail!(IoError::FailedToStripPrefix {
            path: path.clone(),
            root: root.to_path_buf()
        }),
    };

    // Deleting extension
    module_path.set_extension("");

    // Converting to string
    let result = module_path.to_string();

    // Normalizing windows paths
    result.replace("\\", "/")
}

/// Reads file to string
pub fn read(path: &Utf8PathBuf) -> String {
    match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(_) => bail!(IoError::FailedToRead { path: path.clone() }),
    }
}

/// Recursively collects all `.wt` files in directory
pub fn collect_sources(path: &Utf8PathBuf) -> Vec<Utf8PathBuf> {
    // Finding entries with `.wt` extension
    let entries = WalkDir::new(path).into_iter().filter_entry(|e| {
        let entry_path = e.path();
        if entry_path.is_file() {
            match entry_path.extension() {
                Some(ext) => ext == "wt",
                None => false,
            }
        } else {
            true
        }
    });

    // Validating entries
    let mut result: Vec<Utf8PathBuf> = Vec::new();
    for entry in entries {
        match entry {
            Ok(path) => {
                // Getting &Path
                let path = path.path();

                // Skipping directories
                if path.is_dir() {
                    continue;
                }

                // Converting path buf into utf-8 path buf
                let utf8_path_result = Utf8PathBuf::from_path_buf(path.to_path_buf());
                match utf8_path_result {
                    Ok(utf8_path) => result.push(utf8_path),
                    Err(_) => bail!(IoError::FailedToConvertPathBuf {
                        path: path.to_path_buf()
                    }),
                }
            }
            _ => bail!(IoError::EntryError { path: path.clone() }),
        }
    }

    // Returning result
    result
}
