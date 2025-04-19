use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use crate::errors::{Error, ErrorType};
use crate::lexer::address::Address;
use crate::vm::values::Value;

#[derive(Debug, Clone)]
pub struct Frame {
    map: BTreeMap<String, Value>,
    pub root: Option<Arc<Mutex<Frame>>>,
    pub closure: Option<Arc<Mutex<Frame>>>,
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
            while let Some(ref current_ref) = current.clone() {
                let current_lock = current_ref.lock().unwrap();
                if current_lock.has(name.clone()) {
                    current = current_lock.root.clone();
                }
            }
            false
        }
    }

    pub fn lookup(&self, address: Address, name: String) -> Result<Value, Error> {
        // checking current frame
        if let Some(val) = self.map.get(&name) {
            return Ok(val.clone())
        }
        if let Some(ref closure_ref) = self.closure {
            let closure_lock = closure_ref.lock().unwrap();
            if closure_lock.has(name.clone()) {
                return closure_lock.lookup(address, name);
            }
        }
        // checking others
        let mut current = self.root.clone();
        while let Some(ref current_ref) = current.clone() {
            let current_lock = current_ref.lock().unwrap();
            if current_lock.has(name.clone()) {
                return current_lock.lookup(address.clone(), name.clone())
            }
            current = current_lock.root.clone();
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
        if let Some(val) = self.map.get(&name) {
            self.map.insert(name, val.clone());
            return Ok(());
        }
        if let Some(ref closure_ref) = self.closure {
            let mut closure_lock = closure_ref.lock().unwrap();
            if closure_lock.has(name.clone()) {
                closure_lock.set(address, name, val)?;
                return Ok(());
            }
        }
        // checking others
        let mut current = self.root.clone();
        while let Some(ref current_ref) = current.clone(){
            let mut current_lock = current_ref.lock().unwrap();
            if current_lock.has(name.clone()) {
                current_lock.set(address.clone(), name.clone(), val.clone())?;
            }
            current = current_lock.root.clone();
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