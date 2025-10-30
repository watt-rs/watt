use miette::NamedSource;
/// Imports
use std::{
    ops::{Add, Range},
    sync::Arc,
};

/// Address structure
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Address {
    pub source: Arc<NamedSource<String>>,
    pub span: Range<usize>,
}
/// Address implementation
impl Address {
    /// New address with column
    pub fn new(source: Arc<NamedSource<String>>, at: usize) -> Address {
        Address {
            source,
            span: at..at,
        }
    }
    /// New address with span
    pub fn span(source: Arc<NamedSource<String>>, span: Range<usize>) -> Address {
        Address { source, span }
    }
}
/// Add implementation
impl Add for Address {
    type Output = Address;

    fn add(self, rhs: Self) -> Self::Output {
        if self.source != rhs.source {
            panic!("address sources missmatched.")
        }
        return Address::span(self.source, self.span.start..rhs.span.end);
    }
}
