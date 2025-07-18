// imports
use crate::vm::memory::memory;
use crate::vm::table::Table;
use crate::vm::values::{FnOwner, Value};
use crate::vm::vm::VM;
use std::borrow::Cow;
use rustc_hash::FxHashSet;

/// Garbage collector
///
/// * `objects`: contains all ever allocated values, what alive.
/// * `marked`: contains all marked values during collect_garbage.
/// * `marked_tables`: contains all marked tables during collect_garbage.
/// * `guard`: contains all guarded from garbage collection objects.
/// * `debug`: enable/disable debug messages
///
#[derive(Debug)]
pub struct GC {
    objects: FxHashSet<Value>,
    marked: FxHashSet<Value>,
    marked_tables: FxHashSet<*mut Table>,
    guard: Vec<Value>,
    debug: bool,
}

/// Mark & sweep garbage collector implementation
impl GC {
    /// New gc
    pub fn new(debug: bool) -> GC {
        GC {
            objects: FxHashSet::default(),
            marked: FxHashSet::default(),
            marked_tables: FxHashSet::default(),
            guard: Vec::new(),
            debug,
        }
    }

    /// Prints message by calling
    /// closure, if debug is enabled
    fn log<F: FnOnce() -> Cow<'static, str>>(&self, message: F) {
        if self.debug {
            println!("{}", message());
        }
    }

    /// Resets `marked` and `marked_tables` after garbage collection
    fn reset(&mut self) {
        self.marked.clear();
        self.marked_tables.clear();
    }

    /// Marks value
    ///
    /// Mark will be affected only on the
    /// reference types except Type && Trait
    ///
    #[allow(unused_parens)]
    pub fn mark_value(&mut self, value: Value) {
        // if value is already marked, skip
        if self.marked.contains(&value) {
            return;
        }
        // logging marking value
        self.log(|| Cow::Owned(format!("gc :: mark :: value = {value:?}")));
        // marking reference types
        match value {
            Value::Instance(instance) => unsafe {
                self.mark_table((*instance).fields);
                self.marked.insert(value);
            },
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
            },
            Value::Unit(unit) => unsafe {
                self.mark_table((*unit).fields);
                self.marked.insert(value);
            },
            Value::Native(_) => {
                self.marked.insert(value);
            }
            Value::String(_) => {
                self.marked.insert(value);
            }
            Value::List(list) => unsafe {
                for value in &*list{
                    self.mark_value(*value);
                }
                self.marked.insert(value);
            },
            Value::Any(_) => {
                self.marked.insert(value);
            }
            _ => {}
        }
    }

    /// Marks table
    /// if it's not already marked
    ///
    /// Marks values inside
    ///
    unsafe fn mark_table(&mut self, table: *mut Table) {
        // checking pointer is not null
        if table.is_null() {
            return;
        }
        // if table is already marked, skip
        if self.marked_tables.contains(&table) {
            return;
        }
        // adding to marked list
        self.marked_tables.insert(table);
        // logging marked table
        self.log(|| Cow::Owned(format!("gc :: mark :: table = {table:?}")));
        // marking table values
        for val in (*table).fields.values() {
            self.mark_value(*val);
        }
        // marking table closure
        if !(*table).closure.is_null() {
            self.mark_table((*table).closure);
        }
        // marking table root
        if !(*table).root.is_null() {
            self.mark_table((*table).root);
        }
        // marking table parent
        if !(*table).parent.is_null() {
            self.mark_table((*table).parent);
        }
    }

    /// Sweeps up trash
    /// Freeing unmarked objects during mark phase
    ///
    fn sweep(&mut self) {
        // logging sweep is running
        self.log(|| Cow::Borrowed("gc :: sweep :: running"));
        // finding unmarked objects
        let mut to_free = vec![];
        self.objects.retain(|value| {
            if self.marked.contains(value) {
                true
            } else {
                to_free.push(*value);
                false
            }
        });
        // freeing this objects
        for value in to_free {
            self.free_value(value);
        }
    }

    /// Adding object to allocated list
    /// Necessary for all reference type
    /// values except Type && Trait
    ///
    pub fn add_object(&mut self, value: Value) {
        match value {
            Value::Instance(_)
            | Value::Fn(_)
            | Value::Native(_)
            | Value::String(_)
            | Value::Unit(_)
            | Value::List(_)
            | Value::Any(_) => {
                if !self.objects.contains(&value) {
                    self.objects.insert(value);
                }
            }
            _ => {}
        }
    }

    /// Freeing value
    fn free_value(&self, value: Value) {
        // logging value is freeing
        self.log(|| Cow::Owned(format!("gc :: free :: value = {value:?}")));
        // free
        match value {
            Value::Fn(f) => {
                if !f.is_null() {
                    memory::free_value(f);
                }
            }
            Value::Instance(i) => {
                if !i.is_null() {
                    memory::free_value(i);
                }
            }
            Value::String(s) => {
                if !s.is_null() {
                    memory::free_const_value(s);
                }
            }
            Value::Native(n) => {
                if !n.is_null() {
                    memory::free_value(n);
                }
            }
            Value::Unit(u) => {
                if !u.is_null() {
                    memory::free_value(u);
                }
            }
            Value::List(l) => {
                if !l.is_null() {
                    memory::free_value(l);
                }
            }
            Value::Any(a) => {
                if !a.is_null() {
                    memory::free_value(a);
                }
            }
            _ => {
                println!("unexpected gc value = {value:?}.");
            }
        }
    }

    /// Push value to guard stack
    /// Protects the value from being freed during
    /// sweep phase
    ///
    pub fn push_guard(&mut self, value: Value) {
        self.guard.push(value);
    }

    /// Pop value from the guard stack
    pub fn pop_guard(&mut self) {
        self.guard.pop();
    }

    /// Collect garbage
    /// Collects unused values
    ///
    /// Has medium runtime cost
    ///
    pub unsafe fn collect_garbage(&mut self, vm: &mut VM, table: *mut Table) {
        println!("triggered gc");
        // logging gc is triggered
        self.log(|| Cow::Borrowed("gc :: triggered"));

        // mark phase
        // > stack
        for val in &vm.stack {
            self.mark_value(*val)
        }
        // > units
        self.mark_table(vm.units);
        // > natives
        self.mark_table(vm.natives);
        // > table
        self.mark_table(table);
        // > guard
        for value in self.guard.clone() {
            self.mark_value(value);
        }

        // sweep phase
        self.sweep();

        // reset gc mark vectors
        self.reset();

        // log gc ended
        self.log(|| Cow::Borrowed("gc :: end"));
    }

    /// Allocated values amount
    pub fn objects_amount(&mut self) -> usize {
        self.objects.len()
    }

    /// Full garbage collector cleanup
    /// Freeing all allocated values
    pub fn cleanup(&mut self) {
        // log gc is cleaning up
        self.log(|| Cow::Owned(format!("gc :: cleanup :: {:?}", self.objects.len())));

        // freeing objects
        for value in &self.objects {
            self.free_value(*value);
        }
    }
}
