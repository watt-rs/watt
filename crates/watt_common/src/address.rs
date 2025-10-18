/// Imports
use std::ops::{Add, Range};

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
/// Add implementation
impl Add for Address {
    type Output = Address;

    fn add(self, rhs: Self) -> Self::Output {
        return Address::span(self.span.start..rhs.span.end);
    }
}
