//// Imports
use crate::memory::{memory, trace::Trace};
use crate::{gc_check, guard, unguard};
use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
    ptr::NonNull,
};
/// Garbage collectable value
/// based on refcounter mechanism
#[derive(Debug)]
pub struct Gc<T: Trace + 'static> {
    inner: NonNull<T>,
}

/// Garbage collectable value implementation
impl<T: Trace + 'static> Gc<T> {
    /// New gc value, allocates value in heap
    pub fn new(value: T) -> Self {
        // creating ptr
        let value_ptr = memory::alloc_value(value);
        let result = match NonNull::new(value_ptr) {
            Some(value_ptr_non_null) => Gc {
                inner: value_ptr_non_null,
            },
            None => panic!("NonNull::new returned Option::None."),
        };
        // guarding in gc
        guard!(result);
        // checking gc
        gc_check!();
        // unguarding in gc
        unguard!(result);

        result
    }

    /// New gc value from raw ptr
    #[allow(dead_code)]
    pub fn from_raw(raw: *mut T) -> Gc<T> {
        match NonNull::new(raw) {
            Some(value_ptr_non_null) => Gc {
                inner: value_ptr_non_null,
            },
            None => panic!("NonNull::new returned Option::None."),
        }
    }

    /// As raw
    pub fn raw(&self) -> *mut T {
        self.inner.as_ptr()
    }
}

/// Clone implementation
impl<T: Trace + 'static> Clone for Gc<T> {
    fn clone(&self) -> Gc<T> {
        Self { inner: self.inner }
    }
}

/// Deref implementation
impl<T: Trace + 'static> Deref for Gc<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.raw() }
    }
}

/// DerefMut implementation
impl<T: Trace + 'static> DerefMut for Gc<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.raw() }
    }
}

/// Partialeq implementation
impl<T: Trace + 'static> PartialEq for Gc<T> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.inner.as_ptr(), other.inner.as_ptr())
    }
}

/// Eq implementation
impl<T: Trace + 'static> Eq for Gc<T> {}

/// Hash implementation
impl<T: Trace + 'static> Hash for Gc<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.raw().hash(state);
    }
}
