use std::collections::BTreeMap;
use std::sync::{Arc};
use parking_lot::ReentrantMutex;
use crate::errors::{Error, ErrorType};
use crate::lexer::address::Address;
use crate::{lock};
use crate::vm::utils::SyncCell;
use crate::vm::values::Value;
use crate::vm::vm::ControlFlow;

#[derive(Debug, Clone)]
pub struct Frame {
    pub(crate) map: BTreeMap<String, Value>,
    pub root: Option<SyncCell<Frame>>,
    pub closure: Option<SyncCell<Frame>>,
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
            while let Some(ref mut current_ref) = current.clone() {
                let guard = lock!(current_ref);
                if guard.has(name.clone()) {
                    return true;
                } else {
                    current = guard.root.clone();
                }
            }
            false
        }
    }

    pub fn lookup(&self, address: Address, name: String) -> Result<Value, ControlFlow> {
        // checking current frame
        if let Some(val) = self.map.get(&name) {
            return Ok(val.clone())
        }
        if let Some(ref closure_ref) = self.closure {
            let guard = lock!(closure_ref);
            if guard.has(name.clone()) {
                return guard.lookup(address, name);
            }
        }
        // checking others
        let mut current = self.root.clone();
        while let Some(ref current_ref) = current.clone() {
            let guard = lock!(current_ref);
            if guard.has(name.clone()) {
                return guard.lookup(address.clone(), name.clone())
            }
            current = guard.root.clone();
        }
        // error
        Err(ControlFlow::Error(Error::new(
            ErrorType::Runtime,
            address,
            format!("not found: {:?}", name),
            "check variable existence.".to_string()
        )))
    }
    pub fn find(&self, address: Address, name: String) -> Result<Value, ControlFlow> {
        // checking current frame
        if let Some(val) = self.map.get(&name) {
            return Ok(val.clone())
        }
        // error
        Err(ControlFlow::Error(Error::new(
            ErrorType::Runtime,
            address,
            format!("not found: {:?}", name),
            "check variable existence.".to_string()
        )))
    }

    pub fn set(&mut self, address: Address, name: String, val: Value) -> Result<(), ControlFlow> {
        // checking current frame
        if self.map.contains_key(&name) {
            self.map.insert(name, val.clone());
            return Ok(());
        }
        if let Some(ref mut closure_ref) = self.closure {
            let mut guard = lock!(closure_ref);
            if guard.has(name.clone()) {
                guard.set(address.clone(), name.clone(), val.clone())?;
                return Ok(());
            }
        }
        // checking others
        let mut current = self.root.clone();
        while let Some(ref mut current_ref) = current.clone(){
            let mut guard = lock!(current_ref);
            if guard.has(name.clone()) {
                guard.set(address.clone(), name.clone(), val.clone())?;
                return Ok(());
            }
            current = guard.root.clone();
        }
        // error
        Err(ControlFlow::Error(Error::new(
            ErrorType::Runtime,
            address,
            format!("not found: {:?}", name),
            "check variable existence.".to_string()
        )))
    }

    pub fn set_current(&mut self, address: Address, name: String, val: Value) -> Result<(), ControlFlow> {
        // checking current frame
        if self.map.contains_key(&name) {
            self.map.insert(name, val.clone());
            return Ok(());
        }
        // error
        Err(ControlFlow::Error(Error::new(
            ErrorType::Runtime,
            address,
            format!("not found: {:?}", name),
            "check variable existence.".to_string()
        )))
    }

    pub fn define(&mut self, address: Address, name: String, val: Value) -> Result<(), ControlFlow> {
        // checking current frame
        if self.map.contains_key(&name) {
            self.map.insert(name.clone(), val);
            Err(ControlFlow::Error(Error::new(
                ErrorType::Runtime,
                address,
                format!("already defined: {:?}", name),
                "check variable overrides.".to_string()
            )))
        } else {
            self.map.insert(name, val);
            Ok(())
        }
    }

    pub fn set_root(&mut self, frame: SyncCell<Frame>) {
        // current roo
        if self.root.is_none() {
            self.root = Some(frame.clone());
            return;
        }
        // other roots
        let mut last_root = self.root.clone();
        while last_root.is_some() {
            let root_cloned = last_root.clone().unwrap();
            let guard = lock!(root_cloned);
            let new_root = guard.root.clone();
            if new_root.is_some() {
                last_root = new_root;
            } else {
                break;
            }
        }
        let mut root_cloned = last_root.clone().unwrap();
        let mut guard = lock!(root_cloned);
        guard.root = Option::Some(frame.clone());
    }
}