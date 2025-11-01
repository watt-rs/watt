/// Imports
use miette::NamedSource;
use std::{
    fmt::Debug,
    ops::{Add, Range},
    sync::Arc,
};

/// Address structure
#[derive(Clone, Eq, PartialEq, Hash)]
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

/// Debug implementation
impl Debug for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Address({}..{})", self.span.start, self.span.end)
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
