use std::collections::BTreeMap;
use crate::errors::{Error, ErrorType};
use crate::lexer::address::Address;
use crate::vm::values::Value;

#[derive(Clone, Debug)]
pub struct Table {
    fields: BTreeMap<String, *mut Value>,
    root: *mut Table,
    closure: *mut Table,
}

impl Table {
    pub fn new() -> Table {
        Table {
            fields: BTreeMap::new(),
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

    pub unsafe fn find(&self, address: Address, name: String) -> Result<*mut Value, Error> {
        if self.exists(name.clone()) {
            if self.fields.contains_key(&name) {
                Ok(self.fields[&name])
            } else {
                Ok((*self.closure).find(address, name.clone())?)
            }
        } else {
            Err(Error::new(
                ErrorType::Runtime,
                address,
                name.clone() + " is not found.",
                "check variable existence.".to_string()
            ))
        }
    }

    pub fn define(&mut self, address: Address, name: String, value: *mut Value) -> Result<(), Error> {
        if !self.fields.contains_key(&name) {
            self.fields.insert(name, value);
            Ok(())
        } else {
            Err(Error::new(
                ErrorType::Runtime,
                address,
                name + " is already defined.",
                "you can rename variable.".to_string()
            ))
        }
    }

    pub unsafe fn set(&mut self, address: Address, name: String, value: *mut Value) -> Result<(), Error> {
        let mut current = self as *mut Table;
        while !((*current).fields.contains_key(&name)) {
            if (*current).root.is_null() {
                return Err(Error::new(
                    ErrorType::Runtime,
                    address,
                    name + " is not defined.",
                    "you can define it, using := op.".to_string()
                ))
            }
            current = (*current).root;
        }
        (*current).fields.insert(name, value);
        Ok(())
    }
}