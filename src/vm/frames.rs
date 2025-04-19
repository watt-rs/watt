use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::Arc;
use crate::errors::{Error, ErrorType};
use crate::lexer::address::Address;
use crate::vm::values::Value;

#[derive(Debug, Clone)]
pub struct Frame {
    map: BTreeMap<String, Value>,
    pub root: Option<Arc<RefCell<Frame>>>,
    pub closure: Option<Arc<RefCell<Frame>>>,
}

impl Frame {
    pub fn new() -> Frame {
        Frame {
            map: BTreeMap::new(),
            root: Option::None,
            closure: Option::None
        }
    }

    pub fn has(&self, name: String) -> bool {
        if self.map.contains_key(&name) {
            true
        } else {
            let mut current = self.root.clone();
            while current.is_some() {
                if let Some(ref current_ref) = current {
                    if current_ref.borrow().has(name.clone()) {
                        return true
                    }
                }
                current = current.unwrap().borrow().root.clone();
            }
            false
        }
    }

    pub fn lookup(&self, address: Address, name: String) -> Result<Value, Error> {
        // checking current frame
        if self.map.contains_key(&name) {
            return Ok(self.map.get(&name).unwrap().clone())
        } else {
            if let Some(ref ref_closure) = self.closure {
                if ref_closure.borrow().has(name.clone()) {
                    return Ok(ref_closure.borrow().lookup(address.clone(), name.clone())?)
                }
            }
        }
        // checking others
        let mut current = self.root.clone();
        while current.is_some() {
            if let Some(ref current_ref) = current {
                if current_ref.borrow().has(name.clone()) {
                    return Ok(current_ref.borrow().lookup(address.clone(), name.clone())?)
                }
            }
            current = current.unwrap().borrow().root.clone();
        }
        // error
        Err(Error::new(
            ErrorType::Runtime,
            address,
            format!("not found: {:?}", name),
            "check variable existence.".to_string()
        ))
    }

    pub fn set(&mut self, address: Address, name: String, val: Value) -> Result<(), Error> {
        // checking current frame
        if self.map.contains_key(&name) {
            self.map.insert(name, val);
            return Ok(())
        } else {
            if let Some(ref closure) = self.closure {
                closure.borrow_mut().map.insert(name, val);
                return Ok(())
            }
        }
        // checking others
        let mut current = self.root.clone();
        while current.is_some() {
            if let Some(ref current_ref) = current {
                if current_ref.borrow().has(name.clone()) {
                    current_ref.borrow_mut().map.insert(name.clone(), val.clone());
                    return Ok(())
                }
            }
            current = current.unwrap().borrow().root.clone();
        }
        // error
        Err(Error::new(
            ErrorType::Runtime,
            address,
            format!("not found: {:?}", name),
            "check variable existence.".to_string()
        ))
    }

    pub fn define(&mut self, address: Address, name: String, val: Value) -> Result<(), Error> {
        // checking current frame
        if self.map.contains_key(&name) {
            self.map.insert(name.clone(), val);
            Err(Error::new(
                ErrorType::Runtime,
                address,
                format!("already defined: {:?}", name),
                "check variable overrides.".to_string()
            ))
        } else {
            self.map.insert(name, val);
            Ok(())
        }
    }
}