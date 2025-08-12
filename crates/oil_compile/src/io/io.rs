/// Imports
use crate::io::errors::IoError;
use camino::Utf8PathBuf;
use ecow::EcoString;
use oil_common::bail;
use std::fs;
use walkdir::WalkDir;

/// Oil file
#[derive(Debug)]
pub struct OilFile {
    path: Utf8PathBuf,
}

/// Oil file implementation
impl OilFile {
    /// Creates new oil file from path
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
        return self.path.clone();
    }
}

/// Recursivly collects all .oil files in directory
pub fn collect_sources(path: &Utf8PathBuf) -> Vec<OilFile> {
    // Finding entries with .oil extension recursivly
    let entries = WalkDir::new(path.clone()).into_iter().filter_entry(|e| {
        let entry_path = e.path();
        if entry_path.is_file() {
            return match entry_path.extension() {
                Some(ext) => ext == "oil",
                None => false,
            };
        }
        true
    });

    // Validating entrires
    let mut result: Vec<OilFile> = Vec::new();
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
                    Ok(utf8_path) => result.push(OilFile::new(utf8_path)),
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
pub fn module_name(root: &Utf8PathBuf, file: &OilFile) -> EcoString {
    // Getting module local path
    let mut module_path = match file.path().strip_prefix(root.clone()) {
        Ok(ok) => ok.to_path_buf(),
        Err(_) => bail!(IoError::FailedToStripPrefix {
            path: file.path(),
            root: root.clone()
        }),
    };

    // Deleting extension
    module_path.set_extension("");

    // Converting to string
    let result = module_path.to_string();

    // Normalizing windows paths
    result.replace("\\", "/").into()
}
