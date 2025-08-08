/// Imports
use std::ops::Range;
use std::path::PathBuf;

/// Address structure
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Address {
    pub span: Range<usize>,
    pub file: Option<PathBuf>,
}
/// Address implementation
impl Address {
    /// New address with column
    pub fn new(at: usize, file_path: PathBuf) -> Address {
        Address {
            span: at..at,
            file: Some(file_path),
        }
    }
    /// New address with span
    pub fn span(span: Range<usize>, file_path: PathBuf) -> Address {
        Address {
            span: span,
            file: Some(file_path),
        }
    }
    /// Unknown address
    pub fn unknown() -> Address {
        Address {
            span: 0..0,
            file: None,
        }
    }
}
