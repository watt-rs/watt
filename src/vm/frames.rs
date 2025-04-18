use std::collections::BTreeMap;
use crate::errors::{Error, ErrorType};
use crate::lexer::address::Address;
use crate::vm::values::Value;

#[derive(Debug)]
pub struct Frame {
    map: BTreeMap<String, Value>,
    root: Box<Option<Frame>>,
    closure: Box<Option<Frame>>
}

impl Frame {
    pub fn new() -> Frame {
        Frame {
            map: BTreeMap::new(),
            root: Box::new(Option::None),
            closure: Box::new(Option::None)
        }
    }

    pub fn has(&mut self, name: String) -> bool {
        false
    }

    pub fn exists(&mut self, name: String) -> bool{
        if self.map.contains_key(&name) {
            true
        } else {
            self.get_closure().has(name.clone())
        }
    }

    pub fn get_closure(&mut self) -> Option<&mut Frame> {
        self.closure.as_mut().as_mut()
    }

    pub fn lookup(&mut self, address: Address, name: String) -> Result<Value, Error> {
        let mut current = self;
        while !current.exists(name.clone()) {
            match current.root.as_mut() {
                Some(ref mut r) => {
                    current = r;
                }
                None => return Err(Error::new(
                    ErrorType::Runtime,
                    address.clone(),
                    format!("not found: {:?}", name.clone()),
                    "check for var existence.".to_string()
                ))
            }
        }
        if current.map.contains_key(&name) {
            Ok(current.map.get(&name).unwrap().clone())
        } else {
            Ok(current.get_closure().unwrap().lookup(address, name)?)
        }
    }

    pub fn set(&mut self, address: Address, name: String, val: Value) -> Result<(), Error> {
        let mut current = self;
        while !current.exists(name.clone()) {
            match current.root.as_mut() {
                Some(ref mut r) => {
                    current = r;
                }
                None => return Err(Error::new(
                    ErrorType::Runtime,
                    address.clone(),
                    format!("not found: {:?}", name.clone()),
                    "check for var existence.".to_string()
                ))
            }
        }
        if current.map.contains_key(&(name.clone())) {
            current.map.insert(name, val);
        }
        Ok(())
    }

    pub fn force_set(&mut self, address: Address, name: String, val: Value) {
        self.map.insert(name, val);
    }
}