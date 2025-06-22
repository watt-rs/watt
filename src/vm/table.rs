// импорты
use crate::errors::errors::{Error};
use crate::lexer::address::Address;
use crate::vm::values::Value;
use std::collections::HashMap;
use crate::vm::memory::memory;

// таблица
#[derive(Clone, Debug)]
pub struct Table {
    pub fields: HashMap<String,Value>,
    pub root: *mut Table,
    pub closure: *mut Table
}
// имплементация
impl Table {
    pub fn new() -> Table {
        Table {
            fields: HashMap::new(),
            root: std::ptr::null_mut(),
            closure: std::ptr::null_mut(),
        }
    }

    pub unsafe fn exists(&self, name: String) -> bool {
        if self.fields.contains_key(&name) {
            true
        } else if !self.closure.is_null() && (*self.closure).exists(name.clone()) {
            true
        } else {
            false
        }
    }

    pub unsafe fn find(&self, address: Address, name: String) -> Result<Value, Error> {
        if self.exists(name.clone()) {
            if self.fields.contains_key(&name) {
                Ok(self.fields[&name].clone())
            } else {
                Ok((*self.closure).find(address, name.clone())?)
            }
        } else {
            Err(Error::new(
                address,
                name.clone() + " is not found.",
                "check variable existence.".to_string()
            ))
        }
    }

    pub fn define(&mut self, address: Address, name: String, value: Value) -> Result<(), Error> {
        if !self.fields.contains_key(&name) {
            self.fields.insert(name, value);
            Ok(())
        } else {
            Err(Error::new(
                address,
                name.clone() + " is already defined.",
                "you can rename variable.".to_string()
            ))
        }
    }

    pub unsafe fn set(&mut self, address: Address, name: String, value: Value) -> Result<(), Error> {
        let mut current = self as *mut Table;
        while !(*current).fields.contains_key(&name) {
            if (*current).root.is_null() {
                return Err(Error::new(
                    address,
                    name.clone() + " is not defined.",
                    "you can define it, using := op.".to_string()
                ))
            }
            current = (*current).root;
        }
        (*current).fields.insert(name, value);
        Ok(())
    }

    pub unsafe fn set_local(&mut self, address: Address, name: String, value: Value) -> Result<(), Error> {
        if !self.fields.contains_key(&name) {
            return Err(Error::new(
                address,
                name.clone() + " is not defined.",
                "you can define it, using := op.".to_string()
            ))
        }
        self.fields.insert(name, value);
        Ok(())
    }

    pub unsafe fn has(&mut self, name: String) -> bool {
        let mut current = self as *mut Table;
        while !(*current).exists(name.clone()) {
            if (*current).root.is_null() {
                return false
            }
            current = (*current).root;
        }
        true
    }

    pub unsafe fn lookup(&mut self, address: Address, name: String) -> Result<Value, Error> {
        let mut current = self as *mut Table;
        while !(*current).exists(name.clone()) {
            if (*current).root.is_null() {
                return Err(Error::new(
                    address,
                    name + " is not found.",
                    "check variable existence.".to_string()
                ))
            }
            current = (*current).root;
        }
        Ok((*current).find(address, name.clone())?)
    }

    pub unsafe fn set_root(&mut self, root: *mut Table) {
        let mut current = self as *mut Table;
        while !(*current).root.is_null() {
            current = (*current).root;
        }
        (*current).root = root;
    }

    #[allow(unused)]
    pub unsafe fn del_root(&mut self) {
        let mut current = self as *mut Table;
        while !(*current).root.is_null() {
            let new_root = (*current).root;
            if (*new_root).root.is_null() {
                return;
            }
            current = new_root;
        }
    }

    // глубокая очистка
    pub unsafe fn free_fields(&self) {
        let mut to_free = vec![];
        for v in self.fields.values() {
            if !to_free.contains(v) {
                to_free.push(v.clone());
            }
        }
        for val in to_free {
            match val {
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
                Value::Type(t) => {
                    if !t.is_null() { memory::free_value(t); }
                }
                Value::Trait(t) => {
                    if !t.is_null() { memory::free_value(t); }
                }                
                _ => {}
            }
        }
    }
}

// имплементация Send и Sync для трансфера между потоками
unsafe impl Send for Table {}
unsafe impl Sync for Table {}