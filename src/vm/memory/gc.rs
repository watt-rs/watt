// импорты
use crate::vm::table::Table;
use crate::vm::values::{FnOwner, Value};
use crate::vm::vm::VM;
use crate::vm::memory::memory;
use std::collections::{HashSet};

// структура сборщика мусора
#[derive(Debug)]
pub struct GC {
    objects: HashSet<Value>,
    marked: HashSet<Value>,
    marked_tables: HashSet<*mut Table>,
    guard: Vec<Value>,
    debug: bool,
}

// mark & sweep сборщик мусора
impl GC {
    // новый gc
    pub fn new(debug: bool) -> GC {
        GC {
            objects: HashSet::new(),
            marked: HashSet::new(),
            marked_tables: HashSet::new(),
            guard: Vec::new(),
            debug
        }
    }
    // лог
    fn log(&self, message: String) {
        if self.debug { println!("{}", message) };
    }
    // ресет
    fn reset(&mut self) {
        self.marked = HashSet::new();
        self.marked_tables = HashSet::new();
    }
    // маркинг значения
    #[allow(unused_parens)]
    pub fn mark_value(&mut self, value: Value) {
        // проверяем
        if self.marked.contains(&value) {
            return;
        }
        // лог
        self.log(format!("gc :: mark :: value = {:?}", value));
        // маркинг
        match value {
            Value::Instance(instance) => unsafe {
                self.mark_table((*instance).fields);
                self.marked.insert(value);
            }
            Value::Fn(f) => unsafe {
                self.marked.insert(value);
                self.mark_table((*f).closure);
                if (*f).owner.is_some() {
                    match (*f).owner.clone().unwrap() {
                        FnOwner::Unit(unit) => {
                            self.mark_value(Value::Unit(unit));
                        }
                        FnOwner::Instance(unit) => {
                            self.mark_value(Value::Instance(unit));
                        }
                    }
                }
            }
            Value::Unit(unit) => unsafe {
                self.mark_table((*unit).fields);
                self.marked.insert(value);
            }
            Value::Native(_) => {
                self.marked.insert(value);
            }
            Value::String(_) => {
                self.marked.insert(value);
            }
            Value::List(list) => unsafe {
                for value in (*list).clone() {
                    self.mark_value(value);
                }
                self.marked.insert(value);
            }
            _ => {}
        }
    }
    // маркинг таблицы
    unsafe fn mark_table(&mut self, table: *mut Table) {
        // проверка на нулл
        if table.is_null() { return; }
        // проверяем
        if self.marked_tables.contains(&table) {
            return;
        }
        // добавляем
        self.marked_tables.insert(table);
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
        // маркинг рут таблицы
        if !(*table).root.is_null() {
            self.mark_table((*table).root);
        }
        // маркинг parent таблицы
        if !(*table).parent.is_null() {
            self.mark_table((*table).parent);
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
                to_free.push(*value);
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
        // добавляем
        match value {
            Value::Instance(_) | Value::Fn(_) |
            Value::Native(_) | Value::String(_) |
            Value::Unit(_) | Value::List(_) => {
                if !self.objects.contains(&value) {
                    self.objects.insert(value);
                }
            }
            _ => {}
        }
    }
    // высвобождение значения
    fn free_value(&self, value: Value) {
        self.log(format!("gc :: free :: value = {:?}", value));
        match value {
            Value::Fn(f) => {
                if !f.is_null() { memory::free_value(f); }
            }
            Value::Instance(i) => {
                if !i.is_null() { memory::free_value(i); }
            }
            Value::String(s) => {
                if !s.is_null() { memory::free_const_value(s); }
            }
            Value::Native(n) => {
                if !n.is_null() { memory::free_value(n); }
            }
            Value::Unit(u) => {
                if !u.is_null() { memory::free_value(u); }
            }
            Value::List(l) => {
                if !l.is_null() { memory::free_value(l); }
            }
            _ => {
                println!("unexpected gc value = {:?}.", value);
            }
        }
    }
    // пуш значения в защиту
    pub fn push_guard(&mut self, value: Value) {
        self.guard.push(value);
    }
    // поп значения из защиту
    pub fn pop_guard(&mut self) {
        self.guard.pop();
    }
    // сборка мусора
    pub unsafe fn collect_garbage(&mut self, vm: &mut VM, table: *mut Table) {
        // лог
        self.log("gc :: triggered".to_string());
        // марк
        // > stack
        for val in vm.stack.clone() {
            self.mark_value(val)
        };
        // > units
        self.mark_table(vm.units);
        // > table
        self.mark_table(table);
        // > guard
        for value in self.guard.clone() {
            self.mark_value(value);
        }
        // sweep
        self.sweep();
        // ресет
        self.reset();
        // лог
        self.log("gc :: end".to_string());
    }
    // количество объектов
    pub fn objects_amount(&mut self) -> usize {
        // возвращаем
        self.objects.len()
    }
    // полный cleanup
    pub fn cleanup(&mut self) {
        // лог
        self.log(format!("gc :: cleanup :: {:?}", self.objects.len()));
        // перебираем, и высвобождаем аллоцированые объекты
        for value in &self.objects {
            self.free_value(value.clone());
        }
    }
}