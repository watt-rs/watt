use std::sync::{Arc, Mutex};
use crate::errors::{Error, ErrorType};
use crate::lexer::address::Address;
use crate::vm::bytecode::Chunk;
use crate::vm::frames::Frame;
use crate::vm::vm::{ControlFlow, Vm};

// native
pub type Native = fn(&mut Vm, Address, bool) -> Result<(), ControlFlow>;

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
    pub fn new(vm: &mut Vm, typo: Arc<Mutex<Type>>) -> Result<Instance, ControlFlow> {
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

// function owner
#[derive(Debug, Clone)]
pub enum FunctionOwner {
    Type(Arc<Mutex<Type>>),
    Instance(Arc<Mutex<Instance>>),
}

// function
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub full_name: String,
    pub body: Chunk,
    pub params: Vec<String>,
    pub closure: Arc<Mutex<Frame>>,
    pub owner: FunctionOwner
}

impl Function {
    pub fn new(name: String, full_name: String, body: Chunk, params: Vec<String>, closure: Arc<Mutex<Frame>>, owner: FunctionOwner) -> Function {
        Function {
            name,
            full_name,
            body,
            params,
            closure,
            owner
        }
    }

    pub fn run(&mut self, vm: &mut Vm, address: Address, frame: Arc<Mutex<Frame>>, should_push: bool) -> Result<(), ControlFlow> {
        if let Err(control_flow) = vm.run(self.body.clone(), frame) {
            if let ControlFlow::Return(returnable) = control_flow {
                if should_push {
                    vm.push(address.clone(), returnable.clone())?;
                }
            }
        }
        Ok(())
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
    Unit(Arc<Mutex<Unit>>),
    Type(Arc<Mutex<Type>>),
    Native(Native),
    Fn(Arc<Mutex<Function>>),
    Null,
}

impl Value {
    #[inline]
    pub fn bool(value: Value) -> bool {
        if let Value::Bool(b) = value {
            b
        } else {
            panic!("couldn't transfer: {:?} to bool.", value)
        }
    }
    pub fn eq(&self, address: Address, other: Value) -> bool {
        match (self, other) {
            (Value::Integer(current), Value::Integer(another)) => {
                *current == another
            }
            (Value::Integer(current), Value::Float(another)) => {
                (*current as f64) == another
            }
            (Value::Float(current), Value::Integer(another)) => {
                *current == (another as f64)
            }
            (Value::Float(current), Value::Float(another)) => {
                *current == another
            }
            (Value::Bool(current), Value::Bool(another)) => {
                *current == another
            }
            (Value::String(current), Value::String(another)) => {
                *current == another
            }
            (Value::Instance(current), Value::Instance(another)) => {
                Arc::ptr_eq(&(*current), &another)
            }
            (Value::Unit(current), Value::Unit(another)) => {
                Arc::ptr_eq(&(*current), &another)
            }
            (Value::Fn(current), Value::Fn(another)) => {
                Arc::ptr_eq(&(*current), &another)
            }
            (Value::Type(current), Value::Type(another)) => {
                Arc::ptr_eq(&(*current), &another)
            }
            (Value::Native(current), Value::Native(another)) => {
                *current == another
            }
            (Value::Null, Value::Null) => true,
            _ => false,
        }
    }
    pub fn not_eq(&self, address: Address, other: Value) -> bool {
        !self.eq(address, other)
    }
    pub fn greater(&self, address: Address, other: Value) -> Result<Value, ControlFlow> {
        match (self, other.clone()) {
            (Value::Integer(current), Value::Integer(another)) => {
                Ok(Value::Bool(*current > another))
            }
            (Value::Integer(current), Value::Float(another)) => {
                Ok(Value::Bool((*current as f64) > another))
            }
            (Value::Float(current), Value::Integer(another)) => {
                Ok(Value::Bool(*current > (another as f64)))
            }
            (Value::Float(current), Value::Float(another)) => {
                Ok(Value::Bool(*current > another))
            }
            _ => Err(ControlFlow::Error(Error::new(
                ErrorType::Runtime,
                address,
                format!("couldn't use > for {:?} and {:?}", self, other),
                "check your code.".to_string()
            )))
        }
    }
    pub fn less(&self, address: Address, other: Value) -> Result<Value, ControlFlow> {
        match (self, other.clone()) {
            (Value::Integer(current), Value::Integer(another)) => {
                Ok(Value::Bool(*current < another))
            }
            (Value::Integer(current), Value::Float(another)) => {
                Ok(Value::Bool((*current as f64) < another))
            }
            (Value::Float(current), Value::Integer(another)) => {
                Ok(Value::Bool(*current < (another as f64)))
            }
            (Value::Float(current), Value::Float(another)) => {
                Ok(Value::Bool(*current < another))
            }
            _ => Err(ControlFlow::Error(Error::new(
                ErrorType::Runtime,
                address,
                format!("couldn't use > for {:?} and {:?}", self, other),
                "check your code.".to_string()
            )))
        }
    }

    pub fn greater_eq(&self, address: Address, other: Value) -> Result<Value, ControlFlow> {
        Ok(Value::Bool(
            self.eq(address.clone(), other.clone()) || Value::bool(self.greater(address, other)?))
        )
    }

    pub fn less_eq(&self, address: Address, other: Value) -> Result<Value, ControlFlow> {
        Ok(Value::Bool(
            self.eq(address.clone(), other.clone()) || Value::bool(self.less(address, other)?))
        )
    }
}