/// Imports
use std::ops::Range;

/// Address structure
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Address {
    pub span: Range<usize>,
}
/// Address implementation
impl Address {
    /// New address with column
    pub fn new(at: usize) -> Address {
        Address { span: at..at }
    }
    /// New address with span
    pub fn span(span: Range<usize>) -> Address {
        Address { span }
    }
}
