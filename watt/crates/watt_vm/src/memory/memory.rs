/// Allocates value in heap using box
///
/// returns raw pointer
///
pub fn alloc_value<T>(value: T) -> *mut T {
    Box::into_raw(Box::new(value))
}

/// Frees ptr value using box
pub fn free_value<T: ?Sized>(ptr: *mut T) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(ptr));
    }
}
