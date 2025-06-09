use crate::vm::memory;
use crate::vm::values::Value;
use crate::vm::vm::VM;

// структура сборщика мусора
struct GC {
    allocated_bytes: usize,
    allocated: Vec<*mut Value>,
    marked: Vec<*mut Value>,
}

// mark & sweep сборщик мусора
impl GC {
    fn reset(&mut self) {
        self.marked = vec![];
    }
    fn mark(&mut self, value: *mut Value) {
        self.marked.push(value);
    }
    fn mark_all(vm: &mut VM) {

    }
    fn sweep(&mut self) {
        for value in self.allocated.clone() {
            if !self.marked.contains(&value) {
                memory::free_value(value);
            }
        }
    }
}