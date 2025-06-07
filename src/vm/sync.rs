use std::sync::{Arc, Mutex};

// обёртка над Arc<Mutex<T>>
#[derive(Clone, Debug)]
pub struct SyncCell<T> {
    pub value: Arc<Mutex<T>>
}