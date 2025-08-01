use crate::address::Address;
use std::{fs, path::PathBuf};

/// Error
pub enum FileReadError {
    FileNotFound,
    IoError,
}

/// Reading file
///
/// raises error if path is not exists,
/// or file can not be read.
///
pub fn read_file(addr: Option<Address>, path: &PathBuf) -> Result<String, FileReadError> {
    // if path doesn't exist, we take the directory path of our program that imports the file
    let path: PathBuf = {
        if path.exists() {
            path.to_owned()
        } else if let Some(address) = &addr
            && let Some(file_path) = &address.file
        {
            match file_path.parent() {
                None => return Err(FileReadError::FileNotFound),
                Some(parent) => {
                    let mut result = parent.to_path_buf();
                    result.push(path);
                    if result.exists() {
                        result
                    } else {
                        return Err(FileReadError::FileNotFound);
                    }
                }
            }
        } else {
            return Err(FileReadError::FileNotFound);
        }
    };

    // reading file
    if path.exists() {
        if let Ok(result) = fs::read_to_string(&path) {
            Ok(result)
        } else {
            return Err(FileReadError::IoError);
        }
    } else {
        return Err(FileReadError::IoError);
    }
}

// Creates default full_name_prefix
pub fn delete_extension(file_name: &str) -> String {
    match file_name.rfind(".") {
        Some(index) => file_name[..index].to_string(),
        None => file_name.to_string(),
    }
}
