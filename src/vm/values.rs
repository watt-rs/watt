use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use crate::errors::Error;
use crate::vm::bytecode::Chunk;
use crate::vm::frames::Frame;
use crate::vm::vm::Vm;

// type
#[derive(Debug, Clone)]
pub struct Type {
    pub name: String,
    pub full_name: String,
    pub body: Chunk
}

impl Type {
    pub fn new(name: String, full_name: String, body: Chunk) -> Type {
        Type {name, full_name, body}
    }
}

// instance
#[derive(Debug)]
pub struct Instance {
    fields: Arc<RefCell<Frame>>,
    typo: Arc<RefCell<Type>>
}

impl Instance {
    pub fn new(vm: &mut Vm, typo: Arc<RefCell<Type>>) -> Result<Instance, Error> {
        let instance = Instance {fields: Arc::new(RefCell::new(Frame::new())), typo: typo.clone()};
        vm.run(typo.borrow().body.clone(), instance.fields.clone())?;
        Ok(instance)
    }
}

// unit
#[derive(Debug)]
pub struct Unit {
    pub name: String,
    pub full_name: String,
    pub fields: Arc<RefCell<Frame>>,
    pub body: Chunk
}

impl Unit {
    pub fn new(name: String, full_name: String, body: Chunk) -> Unit {
        Unit {
            name,
            full_name,
            fields: Arc::new(RefCell::new(Frame::new())),
            body
        }
    }
}

// value
#[derive(Clone, Debug)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Instance(Arc<Mutex<Instance>>),
    Null,
}
