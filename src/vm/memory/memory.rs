// импорты
use std::alloc::{alloc, dealloc, Layout};

// аллокация значения в куче
pub fn alloc_value<T: std::fmt::Debug>(value: T) -> *mut T {
    unsafe {
        let layout = Layout::new::<T>();
        let ptr = alloc(layout) as *mut T;
        if ptr.is_null() {
            panic!("alloc failed, pointer = {:?}, layout = {:?}", ptr, layout);
        }
        ptr.write(value);
        ptr
    }
}

// высвобождение памяти
pub fn free_value<T>(ptr: *mut T) {
    unsafe {
        let layout = Layout::new::<T>();
        if ptr.is_null() {
            return;
        }
        ptr.drop_in_place();
        dealloc(ptr as *mut u8, layout);
    }
}

// высвобождение памяти константного указателя
pub fn free_const_value<T>(ptr: *const T) {
    if ptr.is_null() {
        return;
    }
    free_value(ptr as *mut T);
}