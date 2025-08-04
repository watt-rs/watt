use crate::memory::memory;
use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

/// Garbage collectable value
#[derive(Debug)]
struct GcInner<T: ?Sized> {
    ref_count: usize,
    value: *mut T,
}

/// Garbage collectable value
/// based on refcounter mechanism
#[derive(Debug)]
pub struct Gc<T> {
    inner: NonNull<GcInner<T>>,
}

/// Garbage collectable value implementation
impl<T> Gc<T> {
    /// New gc value, allocates value in heap
    pub fn new(value: T) -> Self {
        let value_ptr = memory::alloc_value(value);
        let gc_inner_ptr = memory::alloc_value(GcInner {
            value: value_ptr,
            ref_count: 1,
        });
        match NonNull::new(gc_inner_ptr) {
            Some(gc_inner_non_null) => Gc {
                inner: gc_inner_non_null,
            },
            None => panic!("NonNull::new returned Option::None."),
        }
    }

    /// New gc value from raw ptr
    pub(crate) fn from_raw(raw: *mut T) -> Gc<T> {
        let gc_inner_ptr = memory::alloc_value(GcInner {
            value: raw,
            ref_count: 1,
        });
        match NonNull::new(gc_inner_ptr) {
            Some(gc_inner_non_null) => Gc {
                inner: gc_inner_non_null,
            },
            None => panic!("NonNull::new returned Option::None."),
        }
    }

    /// As raw
    pub fn raw(&self) -> *mut T {
        unsafe { self.inner.as_ref().value }
    }

    /// Strong count
    pub fn strong_count(&self) -> usize {
        unsafe { (*self.inner.as_ptr()).ref_count }
    }

    /// Clone ref
    pub fn clone_ref(&self) -> Self {
        unsafe {
            let inner = self.inner.as_ptr();
            (*inner).ref_count += 1;
        }
        Self { inner: self.inner }
    }
}
/// Clone implementation
impl<T> Clone for Gc<T> {
    fn clone(&self) -> Gc<T> {
        self.clone_ref()
    }
}

/// Drop implementation
impl<T> Drop for Gc<T> {
    fn drop(&mut self) {
        unsafe {
            let inner = self.inner.as_ptr();
            (*inner).ref_count -= 1;
            if (*inner).ref_count == 0 {
                memory::free_value((*inner).value);
                memory::free_value(inner);
            }
        }
    }
}

/// Deref implementation
impl<T> Deref for Gc<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.raw() }
    }
}

/// DerefMut implementation
impl<T> DerefMut for Gc<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.raw() }
    }
}

/// Partialeq implementation
impl<T> PartialEq for Gc<T> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.inner.as_ptr(), other.inner.as_ptr())
    }
}

/// Eq implementation
impl<T> Eq for Gc<T> {}

/// Hash implementation
impl<T> Hash for Gc<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.raw().hash(state);
    }
}
