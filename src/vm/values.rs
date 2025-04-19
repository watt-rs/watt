use std::sync::{Arc, Mutex};
use crate::errors::Error;
use crate::lexer::address::Address;
use crate::vm::bytecode::Chunk;
use crate::vm::frames::Frame;
use crate::vm::vm::Vm;

// native
pub type Native = fn(&mut Vm, Address, Vec<Value>) -> Result<Value, Error>;

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
    pub(crate) fields: Arc<Mutex<Frame>>,
    typo: Arc<Mutex<Type>>
}

impl Instance {
    pub fn new(vm: &mut Vm, typo: Arc<Mutex<Type>>) -> Result<Instance, Error> {
        let instance = Instance {fields: Arc::new(Mutex::new(Frame::new())), typo: typo.clone()};
        vm.run(typo.lock().unwrap().body.clone(), instance.fields.clone())?;
        Ok(instance)
    }
}

// unit
#[derive(Debug)]
pub struct Unit {
    pub name: String,
    pub full_name: String,
    pub fields: Arc<Mutex<Frame>>,
    pub body: Chunk
}

impl Unit {
    pub fn new(name: String, full_name: String, body: Chunk) -> Unit {
        Unit {
            name,
            full_name,
            fields: Arc::new(Mutex::new(Frame::new())),
            body
        }
    }
}

// function
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub full_name: String,
    pub body: Chunk,
    pub params: Vec<String>,
}

// value
#[derive(Clone, Debug)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Instance(Arc<Mutex<Instance>>),
    Unit(Arc<Mutex<Unit>>),
    Type(Arc<Mutex<Type>>),
    Native(Native),
    Fn(Arc<Mutex<Function>>),
    Null,
}
