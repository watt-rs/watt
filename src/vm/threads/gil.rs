// мини gil 🥬
use std::sync::{OnceLock};
use parking_lot::ReentrantMutex;

// сам gil
static GIL: OnceLock<ReentrantMutex<()>> = OnceLock::new();

// его геттер
pub fn get_gil() -> &'static ReentrantMutex<()> {
    GIL.get_or_init(|| ReentrantMutex::new(()))
}

// исполнение с gil'ом
pub fn with_gil<F, R>(f: F) -> R
where F: FnOnce() -> R {
    let _lock = get_gil().lock();
    let _result = f();
    drop(_lock);
    _result
}