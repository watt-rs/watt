// импорты
use crate::vm::threads::threads::Threads;
use crate::vm::vm::VM;

// статичные переменные
pub static mut VM_PTR: Option<*mut VM> = None;
pub static mut THREADS_PTR: Option<*mut Threads> = None;