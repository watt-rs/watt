use std::sync::{Arc, Mutex};
use crate::errors::{Error, ErrorType};
use crate::lexer::address::Address;
use crate::vm::bytecode::Chunk;
use crate::vm::frames::Frame;
use crate::vm::vm::{ControlFlow, Vm};

// native
pub type Native = fn(&mut Vm, Address, bool, i16) -> Result<(), ControlFlow>;

// type
#[derive(Debug, Clone)]
pub struct Type {
    pub name: String,
    pub body: Chunk,
    constructor: Vec<String>
}

impl Type {
    pub fn new(name: String, body: Chunk, constructor: Vec<String>) -> Type {
        Type {name, body, constructor}
    }
}

// instance
#[derive(Debug)]
pub struct Instance {
    pub fields: Arc<Mutex<Frame>>,
    typo: Arc<Mutex<Type>>
}

impl Instance {
    pub fn new(vm: &mut Vm, typo: Arc<Mutex<Type>>, address: Address,
               passed_args: i16, root_frame: Arc<Mutex<Frame>>) -> Result<Arc<Mutex<Instance>>, ControlFlow> {
        let instance = (Arc::new(Mutex::new(Instance {
            fields: Arc::new(Mutex::new(Frame::new())), typo: typo.clone()}
        )));
        let instance_ref = instance.clone();
        let instance_lock = instance_ref.lock().unwrap();
        let typo_lock = instance_lock.typo.lock().unwrap();
        // fields
        let fields_lock_copy = instance_lock.fields.clone();
        let mut fields_lock = fields_lock_copy.lock().unwrap();
        fields_lock.set_root(root_frame.clone());
        // constructor
        let constructor_len = typo_lock.constructor.len() as i16;
        if passed_args != constructor_len {
            return Err(ControlFlow::Error(Error::new(
                ErrorType::Runtime,
                address.clone(),
                format!("couldn't create instance: {:?}, invalid args. ({:?}/{:?})",
                        &instance, passed_args, constructor_len),
                "check your code.".to_string()
            )));
        }
        let mut instance_fields_lock = instance_lock.fields.lock().unwrap();
        for i in (constructor_len-1)..=0 {
            let value = vm.pop(address.clone())?;
            instance_fields_lock.define(
                address.clone(), typo_lock.constructor.get(i as usize).unwrap().clone(), value
            )?
        }
        // body
        vm.run(typo.lock().unwrap().body.clone(), instance_lock.fields.clone())?;
        // fn binds
        for pair in fields_lock.clone().map {
            if let Value::Fn(f) = pair.1 {
                f.lock().unwrap().owner = FunctionOwner::Instance(
                    instance.clone()
                )
            }
        }
        // call init
        if let Some(init) = fields_lock.map.get("init") {

        }
        // return
        Ok(instance)
    }
}

// unit
#[derive(Debug)]
pub struct Unit {
    pub name: String,
    pub fields: Arc<Mutex<Frame>>,
    pub body: Chunk
}

impl Unit {
    pub fn new(vm: &mut Vm, name: String,
               body: Chunk, root_frame: Arc<Mutex<Frame>>) -> Result<Arc<Mutex<Unit>>, ControlFlow> {
        let unit = Arc::new(Mutex::new(Unit {
            name,
            fields: Arc::new(Mutex::new(Frame::new())),
            body
        }));
        let unit_ref = unit.clone();
        let unit_deref = unit_ref.lock().unwrap();
        // fields
        let fields_lock_copy = unit_deref.fields.clone();
        let mut fields_lock = fields_lock_copy.lock().unwrap();
        fields_lock.set_root(root_frame);
        vm.run(unit_deref.body.clone(), unit_deref.fields.clone())?;
        // fn binds
        for pair in fields_lock.clone().map {
            if let Value::Fn(f) = pair.1 {
                f.lock().unwrap().owner = FunctionOwner::Unit(
                    unit.clone()
                )
            }
        }
        // returns unit
        Ok(unit)
    }
}

// function owner
#[derive(Debug, Clone)]
pub enum FunctionOwner {
    Unit(Arc<Mutex<Unit>>),
    Instance(Arc<Mutex<Instance>>),
    NoOne
}

// function
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub body: Chunk,
    pub params: Vec<String>,
    pub closure: Option<Arc<Mutex<Frame>>>,
    pub owner: FunctionOwner
}

impl Function {
    pub fn new(name: String, body: Chunk, params: Vec<String>) -> Function {
        Function {
            name,
            body,
            params,
            closure: None,
            owner: FunctionOwner::NoOne
        }
    }

    pub fn run(&mut self, vm: &mut Vm, address: Address, frame: Arc<Mutex<Frame>>,
               should_push: bool, passed_args: i16) -> Result<(), ControlFlow> {
        let args_len = self.params.len() as i16;
        if passed_args != args_len {
            return Err(ControlFlow::Error(Error::new(
                ErrorType::Runtime,
                address.clone(),
                format!("couldn't call: {:?}, invalid args. ({:?}/{:?})",
                self.name.clone(), passed_args, args_len),
                "check your code.".to_string()
            )));
        }
        let frame_clone = frame.clone();
        let mut frame_lock = frame_clone.lock().unwrap();
        for i in (args_len-1)..=0 {
            let value = vm.pop(address.clone())?;
            frame_lock.define(
                address.clone(), self.params.get(i as usize).unwrap().clone(), value
            )?
        }
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