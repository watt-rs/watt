/// Imports
use camino::Utf8PathBuf;

/// Draft package lints
#[derive(Clone)]
pub struct DraftPackageLints {
    /// Disabled lints
    pub disabled: Vec<String>,
}

/// Draft package
#[derive(Clone)]
pub struct DraftPackage {
    /// Path to package
    pub path: Utf8PathBuf,
    /// Lints config
    pub lints: DraftPackageLints,
}