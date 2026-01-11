/// Imports
use crate::errors::PackageError;
use camino::Utf8PathBuf;
use url::Url;
use watt_common::bail;

/// Converts url to package name
///
/// # Example
/// ```example
//  https://github.com/watt-rs/std -> std
//  https://org.gittea.com/repo -> repo
//  ...
//  ```
//
pub fn url_to_pkg_name(url: &str) -> String {
    match Url::parse(url) {
        Ok(ok) => match ok
            .path_segments()
            .and_then(|mut segments| segments.next_back())
        {
            Some(segment) => match segment.strip_suffix(".git") {
                Some(name) => name.to_string(),
                None => segment.to_string(),
            },
            None => bail!(PackageError::InvalidUrl {
                url: url.to_owned()
            }),
        },
        Err(_) => bail!(PackageError::InvalidUrl {
            url: url.to_owned()
        }),
    }
}

/// Path to package name
///
/// # Example
/// ```example
/// ~/watt/test/ -> test
/// ~/watt/test/.cache/std -> std
/// ...
/// ```
///
pub fn path_to_pkg_name(path: &Utf8PathBuf) -> String {
    match path.file_name() {
        Some(file_name) => file_name.to_string(),
        None => bail!(PackageError::FailedToGetProjectNameFromPath { path: path.clone() }),
    }
}
