/// Imports
use core::fmt;
use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
    rc::Rc,
};

/// Rc that compares by ptr
#[derive(Clone)]
pub struct RcPtr<T>(Rc<T>);

/// Implementation
impl<T> RcPtr<T> {
    /// Creates new RcPtr
    pub fn new(value: T) -> Self {
        RcPtr(Rc::new(value))
    }
}

/// Implementation for T: PartialEq
impl<T: PartialEq + Debug> RcPtr<T> {
    /// Eq by value
    pub fn veq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

/// PartialEq by pointer implementation
impl<T> PartialEq for RcPtr<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

/// Eq implementation
impl<T> Eq for RcPtr<T> {}

/// Deref implementation
impl<T> Deref for RcPtr<T> {
    type Target = Rc<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// DerefMut implementation
impl<T> DerefMut for RcPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Debug implementation
impl<T: fmt::Debug> fmt::Debug for RcPtr<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
