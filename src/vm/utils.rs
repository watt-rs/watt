use std::cell::Cell;
use std::sync::{Arc, Mutex, MutexGuard};

#[macro_export]
macro_rules! lock {
    ($mutex:expr) => {
        $mutex.borrow()
    };
}

#[macro_export]
macro_rules! lock_mut {
    ($mutex:expr) => {
        $mutex.borrow_mut()
    };
}

#[derive(Debug)]
pub struct SyncCell<T> {
     value: Arc<Mutex<T>>
}

impl<T> Clone for SyncCell<T> {
    fn clone(&self) -> Self {
        SyncCell { value: Arc::clone(&self.value) }
    }
}

impl<T> SyncCell<T> {
    pub fn new(value: T) -> Self {
        SyncCell { value: Arc::new(Mutex::new(value))}
    }

    pub fn borrow(&self) -> MutexGuard<T> {
        match self.value.try_lock() {
            Ok(guard) => guard,
            Err(_) => panic!("sync cell already locked. drop the MutexGuard, before lock it secondly."),
        }
    }

    pub fn equals(&self, other: &SyncCell<T>) -> bool {
        Arc::ptr_eq(&self.value, &other.value)
    }
}