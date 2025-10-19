/// Modules
pub mod errors;

/// Imports
use crate::io::errors::IoError;
use camino::{Utf8Path, Utf8PathBuf};
use ecow::EcoString;
use log::info;
use std::{
    ffi::OsStr,
    fs::{self, File},
    io::Write,
};
use walkdir::WalkDir;
use watt_common::bail;

/// Watt file
#[derive(Debug)]
pub struct WattFile {
    path: Utf8PathBuf,
}

/// Watt file implementation
impl WattFile {
    /// Creates new watt file from path
    pub fn new(path: Utf8PathBuf) -> Self {
        Self { path }
    }

    /// Safely reads file
    pub fn read(&self) -> String {
        match fs::read_to_string(&self.path) {
            Ok(text) => text,
            Err(_) => bail!(IoError::FailedToRead {
                path: self.path.clone()
            }),
        }
    }

    /// Gets path clone
    pub fn path(&self) -> Utf8PathBuf {
        self.path.clone()
    }
}

/// Recursivly collects all .watt files in directory
pub fn collect_sources(path: &Utf8PathBuf) -> Vec<WattFile> {
    // Finding entries with .watt extension recursivly
    let entries = WalkDir::new(path.clone()).into_iter().filter_entry(|e| {
        let entry_path = e.path();
        if entry_path.is_file() {
            match entry_path.extension() {
                Some(ext) => ext == "wt",
                None => false,
            }
        } else {
            // Ignoring .cache directory
            match entry_path.file_name() {
                Some(directory) => directory != OsStr::new(".cache"),
                None => false,
            }
        }
    });

    // Validating entrires
    let mut result: Vec<WattFile> = Vec::new();
    for entry in entries {
        match entry {
            Ok(path) => {
                // Getting &Path
                let path = path.path();

                // Skipping directories
                if path.is_dir() {
                    continue;
                }

                let utf8_path_result = Utf8PathBuf::from_path_buf(path.to_path_buf());
                match utf8_path_result {
                    Ok(utf8_path) => result.push(WattFile::new(utf8_path)),
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

/// Returns module name by path
pub fn module_name(root: &Utf8Path, file: &WattFile) -> EcoString {
    // Getting module local path
    let mut module_path = match file.path().strip_prefix(root) {
        Ok(ok) => ok.to_path_buf(),
        Err(_) => bail!(IoError::FailedToStripPrefix {
            path: file.path(),
            root: root.to_path_buf()
        }),
    };

    // Deleting extension
    module_path.set_extension("");

    // Converting to string
    let result = module_path.to_string();

    // Normalizing windows paths
    EcoString::from(result.replace("\\", "/"))
}

/// Writes text to the file
pub fn write(path: Utf8PathBuf, text: String) {
    // Creating file, if not exists
    match File::create(&path) {
        Ok(mut file) => {
            // Writing text
            if file.write(&text.into_bytes()).is_err() {
                bail!(IoError::FailedToWrite { path })
            } else {
                info!("Wrote text to {path}")
            }
        }
        Err(_) => {
            bail!(IoError::FailedToWrite { path })
        }
    }
}

/// Creates directory
pub fn mkdir(path: &Utf8PathBuf) {
    // Creating directory, if not exists
    match fs::create_dir(path) {
        Ok(_) => {}
        Err(_) => {
            bail!(IoError::FailedToMkdir { path: path.clone() })
        }
    }
}

/// Creates all directory
pub fn mkdir_all(path: &Utf8PathBuf) {
    // Creating directory, if not exists
    match fs::create_dir_all(path) {
        Ok(_) => {}
        Err(_) => {
            bail!(IoError::FailedToMkdirAll { path: path.clone() })
        }
    }
}
