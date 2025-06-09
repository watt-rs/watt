use crate::vm::{gil, memory};
use crate::vm::table::Table;
use crate::vm::values::{FnOwner, Value};
use crate::vm::vm::VM;

// структура сборщика мусора
#[derive(Debug)]
pub struct GC {
    objects: Vec<Value>,
    marked: Vec<Value>,
    marked_tables: Vec<*mut Table>,
    debug: bool,
}

// mark & sweep сборщик мусора
impl GC {
    // новый gc
    pub fn new(debug: bool) -> GC {
        GC {
            objects: vec![],
            marked: vec![],
            marked_tables: vec![],
            debug
        }
    }
    // лог
    fn log(&self, message: String) {
        if self.debug { println!("{}", message) };
    }
    // ресет
    fn reset(&mut self) {
        self.marked = vec![];
    }
    // маркинг значения
    #[allow(unused_parens)]
    fn mark_value(&mut self, value: Value) {
        // проверяем
        if self.marked.contains(&value) {
            return;
        }
        // лог
        self.log(format!("gc :: mark :: value = {:?}", value));
        // маркинг
        match value {
            Value::Instance(instance) => unsafe {
                self.marked.push(value);
                self.mark_table((*instance).fields);
            }
            Value::Fn(f) => unsafe {
                self.marked.push(value);
                self.mark_table((*f).closure);
                if !(*f).owner.is_null() {
                    match (*(*f).owner) {
                        FnOwner::Unit(unit) => unsafe {
                            self.mark_value(Value::Unit(unit));
                        }
                        FnOwner::Instance(unit) => unsafe {
                            self.mark_value(Value::Instance(unit));
                        }
                    }
                }
            }
            Value::Unit(unit) => unsafe {
                self.marked.push(value);
                self.mark_table((*unit).fields)
            }
            Value::Native(_) => {
                self.marked.push(value);
            }
            Value::String(_) => {
                self.marked.push(value);
            }
            _ => {}
        }
    }
    // маркинг таблицы
    unsafe fn mark_table(&mut self, table: *mut Table) {
        // проверяем
        if self.marked_tables.contains(&table) {
            return;
        }
        // добавляем
        self.marked_tables.push(table);
        // лог
        self.log(format!("gc :: mark :: table = {:?}", table));
        // значения таблицы
        for val in (*table).fields.values() {
            self.mark_value(*val);
        }
        // маркинг замыкания
        if !(*table).closure.is_null() {
            self.mark_table((*table).closure);
        }
        // маркинг таблицы
        if !(*table).root.is_null() {
            self.mark_table((*table).root);
        }
    }
    // очистка
    fn sweep(&mut self) {
        // лог
        self.log("gc :: sweep :: running".to_string());
        // ищем объекты для очистки, и удаляем из списка self.objects
        let mut to_free = vec![];
        self.objects.retain(|value| {
            if self.marked.contains(&value.clone()) {
                true
            } else {
                to_free.push(value.clone());
                false
            }
        });
        // перебираем, и высвобождаем память
        for value in to_free {
            self.free_value(value.clone());
        }
    }
    // добавить в аллоцированные
    pub fn add_object(&mut self, value: Value) {
        match value {
            Value::Instance(_) | Value::Fn(_) |
            Value::Native(_) | Value::Unit(_)  |
            Value::String(_) => {
                if !self.objects.contains(&value) {
                    self.objects.push(value);
                }
            }
            _ => {}
        }
    }
    // очистка
    fn free_value(&self, value: Value) {
        match value {
            Value::Fn(f) => {
                memory::free_value(f);
            }
            Value::Unit(u) => {
                memory::free_value(u);
            }
            Value::Instance(i) => {
                memory::free_value(i);
            }
            Value::Type(t) => {
                memory::free_value(t);
            }
            Value::String(s) => {
                memory::free_const_value(s);
            }
            Value::Native(native) => {
                memory::free_value(native);
            }
            _ => {
                println!("unexpected gc value = {:?}.", value);
            }
        }
        if self.debug { println!("gc :: free :: value = {:?}", value); }
    }
    // сборка мусора
    pub unsafe fn collect_garbage(&mut self, table: *mut Table) {
        // через gil
        gil::with_gil(|| {
            // лог
            self.log("gc :: triggered".to_string());
            // марк
            VM::stack.with(|stack| {
                for val in stack.borrow().iter().cloned() {
                    self.mark_value(val)
                }
            });
            self.mark_table(table);
            // sweep
            self.sweep();
            // ресет
            self.reset()
        });
    }
    // количество объектов
    pub fn objects_amount(&mut self) -> usize {
        self.objects.len()
    }
}