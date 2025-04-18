use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use crate::errors::Error;
use crate::vm::bytecode::Chunk;
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
    fields: BTreeMap<String, Box<Value>>,
    typo: Type
}

impl Instance {
    pub fn new(vm: &mut Vm, typo: Type) -> Result<Instance, Error> {
        vm.run(typo.body.clone())?;
        Ok(Instance {fields: BTreeMap::new(), typo})
    }
}

// unit
#[derive(Debug)]
pub struct Unit {
    pub name: String,
    pub full_name: String,
    fields: BTreeMap<String, Box<Value>>,
    pub body: Chunk
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
