
use std::sync::{Arc};
use parking_lot::ReentrantMutex;
use crate::errors::{Error, ErrorType};
use crate::lexer::address::Address;
use crate::{lock};
use crate::vm::bytecode::Chunk;
use crate::vm::frames::Frame;
use crate::vm::utils::SyncCell;
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
    pub fields: SyncCell<Frame>,
    typo: SyncCell<Type>
}

impl Instance {
    pub fn new(vm: &mut Vm, typo: SyncCell<Type>, address: Address,
               passed_args: i16, root_frame: SyncCell<Frame>) -> Result<SyncCell<Instance>, ControlFlow> {
        let instance = (SyncCell::new(Instance {
            fields: SyncCell::new(Frame::new()), typo: typo.clone()}
        ));
        // guards
        let mut instance_arc = instance.clone();
        let mut instance_guard = lock!(instance_arc);
        let type_binding = instance_guard.typo.clone();
        let type_guard = lock!(type_binding); // constructor
        let mut fields_binding = instance_guard.fields.clone();
        let mut fields_guard = lock!(fields_binding);
        let constructor_len = type_guard.constructor.len() as i16;
        // logic
        if passed_args != constructor_len {
            return Err(ControlFlow::Error(Error::new(
                ErrorType::Runtime,
                address.clone(),
                format!("couldn't create instance: {:?}, invalid args. ({:?}/{:?})",
                        &instance, passed_args, constructor_len),
                "check your code.".to_string()
            )));
        }
        for i in (constructor_len-1)..=0 {
            let value = vm.pop(address.clone())?;
            fields_guard.define(
                address.clone(), type_guard.constructor.get(i as usize).unwrap().clone(), value
            )?
        }
        // body
        vm.run(type_guard.body.clone(), instance_guard.fields.clone())?;
        // fn binds
        let fn_binds_binding = instance_guard.fields.clone();
        let fn_binds_guard = lock!(fn_binds_binding);
        for pair in fn_binds_guard.clone().map {
            if let Value::Fn(mut f) = pair.1 {
                let mut locked = lock!(f);
                locked.owner = FunctionOwner::Instance(
                    instance.clone()
                )
            }
        }
        // call init
        if let Some(some) = fields_guard.map.get("init") {
            if let Value::Fn(ref init_fn) = some {
                let mut local_frame = SyncCell::new(Frame::new());
                let mut local_frame_guard = lock!(local_frame);
                local_frame_guard.set_root(instance_guard.fields.clone());
                lock!(init_fn).run(vm, address, local_frame.clone(), false, 0)?;
            }
        }
        // return
        Ok(instance)
    }
}

// unit
#[derive(Debug, Clone)]
pub struct Unit {
    pub name: String,
    pub fields: SyncCell<Frame>,
    pub body: Chunk
}

impl Unit {
    pub fn new(vm: &mut Vm, name: String,
               body: Chunk, root_frame: SyncCell<Frame>) -> Result<SyncCell<Unit>, ControlFlow> {
        let unit = SyncCell::new(Unit {
            name,
            fields: SyncCell::new(Frame::new()),
            body
        });
        let unit_ref = unit.clone();
        let unit_deref = lock!(unit_ref);
        // fields
        let mut fields_lock_copy = unit_deref.fields.clone();
        let mut fields_lock = lock!(fields_lock_copy);
        fields_lock.set_root(root_frame);
        vm.run(unit_deref.body.clone(), unit_deref.fields.clone())?;
        // fn binds
        for pair in fields_lock.clone().map {
            if let Value::Fn(mut f) = pair.1 {
                lock!(f).owner = FunctionOwner::Unit(
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
    Unit(SyncCell<Unit>),
    Instance(SyncCell<Instance>),
    NoOne
}

// function
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub body: Chunk,
    pub params: Vec<String>,
    pub closure: Option<SyncCell<Frame>>,
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

    pub fn run(&self, vm: &mut Vm, address: Address, frame: SyncCell<Frame>,
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
        let mut working_frame = frame.clone();
        let mut frame_lock = lock!(working_frame);
        for i in (args_len-1)..=0 {
            let value = vm.pop(address.clone())?;
            frame_lock.define(
                address.clone(), self.params.get(i as usize).unwrap().clone(), value
            )?
        }
        drop(frame_lock);
        if let Err(control_flow) = vm.run(self.body.clone(), frame.clone()) {
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
    Instance(SyncCell<Instance>),
    Unit(SyncCell<Unit>),
    Type(SyncCell<Type>),
    Native(SyncCell<Native>),
    Fn(SyncCell<Function>),
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
                current.equals(&another)
            }
            (Value::Unit(current), Value::Unit(another)) => {
                current.equals(&another)
            }
            (Value::Fn(current), Value::Fn(another)) => {
                current.equals(&another)
            }
            (Value::Type(current), Value::Type(another)) => {
                current.equals(&another)
            }
            (Value::Native(current), Value::Native(another)) => {
                current.equals(&another)
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