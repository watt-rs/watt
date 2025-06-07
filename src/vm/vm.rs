use crate::errors::*;
use crate::lexer::address::Address;
use crate::vm::bytecode::*;
use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc};
use crate::{lock};
use crate::vm::frames::Frame;
use crate::vm::values::{Function, Instance, Native, Type, Unit, Value};
use super::bytecode::Opcode;
use parking_lot::ReentrantMutex;
use crate::vm::utils::SyncCell;

// vm struct
pub struct Vm {
    eval_stack: VecDeque<Value>,
    natives: BTreeMap<String, Native>,
    types: BTreeMap<String, SyncCell<Type>>,
    units: BTreeMap<String, SyncCell<Unit>>,
}

// control flow
#[derive(Clone)]
pub enum ControlFlow {
    Return(Value),
    Continue,
    Break,
    Error(Error)
}

// vm impl
impl Vm {
    // new vm instance
    pub fn new() -> Self {
        Vm {
            eval_stack: VecDeque::new(),
            natives: BTreeMap::from([
                ("println".to_string(), |vm: &mut Vm, address: Address, should_push: bool, args_amount:i16| -> Result<(), ControlFlow> {
                    let value = vm.pop(address)?;
                    println!("{:?}", value);
                    return Ok(());
                } as Native)
            ]),
            types: BTreeMap::new(),
            units: BTreeMap::new(),
        }
    }

    // push to stack
    // noinspection RsLiveness
    pub fn push(&mut self, address: Address, value: Value) -> Result<(), ControlFlow> {
        self.eval_stack.push_back(value);
        Ok(())
    }

    pub fn pop(&mut self, address: Address) -> Result<Value, ControlFlow> {
        match self.eval_stack.pop_back() {
            Some(v) => Ok(v),
            None => Err(ControlFlow::Error(Error::new(
                ErrorType::Runtime,
                address,
                "stack is empty.".to_string(),
                "check your code.".to_string(),
            ))),
        }
    }

    pub fn run(&mut self, chunk: Chunk, mut frame: SyncCell<Frame>) -> Result<(), ControlFlow> {
        for op in chunk.opcodes() {
            // println!("running {:?}", op);
            match op {
                // push
                Opcode::Push {
                    addr: address,
                    value,
                } => {
                    self.push(address.clone(), value.clone())?;
                }
                // pop
                Opcode::Pop { addr: address } => {
                    self.pop(address.clone())?;
                }
                // binary
                Opcode::Bin { addr: address, op } => {
                    let left = self.pop(address.clone())?;
                    let right = self.pop(address.clone())?;
                    let number_error = Err(ControlFlow::Error(Error::new(
                        ErrorType::Runtime,
                        address.clone(),
                        format!(
                            "couldn't use {:?} with: {:?} and {:?}",
                            op.clone(),
                            left,
                            right
                        ),
                        "check your code.".to_string(),
                    )));

                    match (left, op.clone().as_str()) {
                        (Value::Integer(l), "+") => match right {
                            Value::Integer(r) => self.push(address.clone(), Value::Integer(l + r))?,
                            Value::Float(r) => self.push(address.clone(), Value::Float((l as f64) + r))?,
                            _ => return number_error.clone()
                        }
                        (Value::Integer(l), "-") => match right {
                            Value::Integer(r) => self.push(address.clone(), Value::Integer(l - r))?,
                            Value::Float(r) => self.push(address.clone(), Value::Float((l as f64) - r))?,
                            _ => return number_error.clone()
                        }
                        (Value::Integer(l), "*") => match right {
                            Value::Integer(r) => self.push(address.clone(), Value::Integer(l * r))?,
                            Value::Float(r) => self.push(address.clone(), Value::Float((l as f64) * r))?,
                            _ => return number_error.clone()
                        }
                        (Value::Integer(l), "/") => match right {
                            Value::Integer(r) => self.push(address.clone(), Value::Integer(l / r))?,
                            Value::Float(r) => self.push(address.clone(), Value::Float((l as f64) / r))?,
                            _ => return number_error.clone()
                        }
                        (Value::Float(l), "+") => match right {
                            Value::Integer(r) => self.push(address.clone(), Value::Float(l + (r as f64)))?,
                            Value::Float(r) => self.push(address.clone(), Value::Float(l + r))?,
                            _ => return number_error.clone()
                        }
                        (Value::Float(l), "-") => match right {
                            Value::Integer(r) => self.push(address.clone(), Value::Float(l - (r as f64)))?,
                            Value::Float(r) => self.push(address.clone(), Value::Float(l - r))?,
                            _ => return number_error.clone()
                        }
                        (Value::Float(l), "*") => match right {
                            Value::Integer(r) => self.push(address.clone(), Value::Float(l * (r as f64)))?,
                            Value::Float(r) => self.push(address.clone(), Value::Float(l * r))?,
                            _ => return number_error.clone()
                        }
                        (Value::Float(l), "/") => match right {
                            Value::Integer(r) => self.push(address.clone(), Value::Float(l / (r as f64)))?,
                            Value::Float(r) => self.push(address.clone(), Value::Float(l / r))?,
                            _ => return number_error.clone()
                        }
                        (Value::String(l), "+") => {
                            match right {
                                Value::String(r) => self.push(address, Value::String(
                                    format!("{:?}{:?}", l, r)
                                ))?,
                                _ => return Err(ControlFlow::Error(Error::new(
                                        ErrorType::Runtime,
                                        address,
                                        format!("couldn't add string with: {:?}", right),
                                        "check your code.".to_string(),
                                    )))
                            }
                        }
                        _ => {}
                    }
                }
                // load
                Opcode::Load { addr: address, name, has_previous, should_push } => {
                    if has_previous {
                        let previous = self.pop(address.clone())?.clone();
                        match previous {
                            Value::Instance(instance) => {
                                let guard = lock!(instance);
                                let fields_guard = lock!(guard.fields);
                                if should_push {
                                    self.push(address.clone(), fields_guard.find(address.clone(), name.clone())?)?
                                }
                            }
                            Value::Unit(unit) => {
                                let guard = lock!(unit);
                                let fields_guard = lock!(guard.fields);
                                if should_push {
                                    self.push(address.clone(), fields_guard.find(address.clone(), name.clone())?)?
                                }
                            }
                            _ => {
                                return Err(ControlFlow::Error(Error::new(
                                    ErrorType::Runtime,
                                    address.clone(),
                                    format!("couldn't load var from: {:?}", previous),
                                    "check your code.".to_string(),
                                )));
                            }
                        }
                    } else {
                        if should_push {
                            let guard = lock!(frame);
                            if guard.has(name.clone()) {
                                self.push(address.clone(), guard.lookup(address.clone(), name.clone())?)?;
                            } else if let Some(type_ref) = self.types.get(&name.clone()) {
                                self.push(address.clone(), Value::Type(type_ref.clone()))?;
                            } else if let Some(unit_ref) = self.units.get(&name.clone()) {
                                self.push(address.clone(), Value::Unit(unit_ref.clone()))?;
                            } else {
                                return Err(ControlFlow::Error(Error::new(
                                    ErrorType::Runtime,
                                    address.clone(),
                                    format!("not found: {:?}", name),
                                    "check your code.".to_string(),
                                )));
                            }
                        }
                    }
                }
                // define
                Opcode::Define { addr: address, name, has_previous, value } => {
                    if has_previous {
                        let previous = self.pop(address.clone())?.clone();
                        match previous {
                            Value::Instance(mut instance) => {
                                let mut guard = lock!(instance);
                                let mut fields_guard = lock!(guard.fields);
                                self.run(*value.clone(), frame.clone())?;
                                fields_guard.define(address.clone(), name.clone(), self.pop(address.clone())?)?;
                            }
                            Value::Unit(mut unit) => {
                                let mut guard = lock!(unit);
                                let mut fields_guard = lock!(guard.fields);
                                self.run(*value.clone(), frame.clone())?;
                                fields_guard.define(address.clone(), name.clone(), self.pop(address.clone())?)?;
                            }
                            _ => {
                                return Err(ControlFlow::Error(Error::new(
                                    ErrorType::Runtime,
                                    address.clone(),
                                    format!("couldn't define var to: {:?}", previous),
                                    "check your code.".to_string(),
                                )));
                            }
                        }
                    } else {
                        self.run(*value.clone(), frame.clone())?;
                        let mut guard = lock!(frame);
                        guard.define(address.clone(), name.clone(), self.pop(address.clone())?)?;
                    }
                }
                // set
                Opcode::Set { addr: address, name, has_previous, value } => {
                    if has_previous {
                        let previous = self.pop(address.clone())?.clone();
                        match previous {
                            Value::Instance(instance) => {
                                self.run(*value.clone(), frame.clone())?;
                                let mut guard = lock!(instance);
                                let mut fields_guard = lock!(guard.fields);
                                fields_guard.set_current(address.clone(), name.clone(), self.pop(address.clone())?)?;
                            }
                            Value::Unit(unit) => {
                                self.run(*value.clone(), frame.clone())?;
                                let mut guard = lock!(unit);
                                let mut fields_guard = lock!(guard.fields);
                                fields_guard.set_current(address.clone(), name.clone(), self.pop(address.clone())?)?;
                            }
                            _ => {
                                return Err(ControlFlow::Error(Error::new(
                                    ErrorType::Runtime,
                                    address.clone(),
                                    format!("couldn't set var to: {:?}", previous),
                                    "check your code.".to_string(),
                                )));
                            }
                        }
                    } else {
                        self.run(*value.clone(), frame.clone())?;
                        let mut guard = lock!(frame);
                        guard.set(address.clone(), name.clone(), self.pop(address.clone())?)?;
                    }
                }
                // call
                Opcode::Call { addr, name, args, has_previous, should_push} => {
                    // args
                    let before = self.eval_stack.len() as i16;
                    self.run(*args.clone(), frame.clone())?;
                    let after = self.eval_stack.len() as i16;
                    let passed_amount = after - before;
                    // call
                    if has_previous {
                        let previous = self.pop(addr.clone())?.clone();
                        match previous {
                            Value::Instance(instance) => {
                                let mut guard = lock!(instance);
                                let mut fields_guard = lock!(guard.fields);
                                let callee = fields_guard.lookup(addr.clone(), name.clone())?;
                                drop(fields_guard);
                                drop(guard);
                                self.call(callee, addr, frame.clone(), should_push, passed_amount)?;
                            }
                            Value::Unit(unit) => {
                                let mut guard = lock!(unit);
                                let mut fields_guard = lock!(guard.fields);
                                let callee = fields_guard.lookup(addr.clone(), name.clone())?;
                                drop(fields_guard);
                                drop(guard);
                                self.call(callee, addr, frame.clone(), should_push, passed_amount)?;
                            }
                            _ => {
                                return Err(ControlFlow::Error(Error::new(
                                    ErrorType::Runtime,
                                    addr.clone(),
                                    format!("couldn't load var from: {:?}", previous),
                                    "check your code.".to_string(),
                                )));
                            }
                        }
                    } else {
                        // native
                        let native = self.natives.get(&name.clone());
                        if let Some(native_ref) = native {
                            // native
                            native_ref(self, addr.clone(), should_push, passed_amount)?;
                        } else {
                            // frame fn
                            let mut guard = lock!(frame);
                            let value = guard.lookup(addr.clone(), name.clone())?;
                            drop(guard);
                            self.call(value,
                                  addr.clone(), frame.clone(), should_push, passed_amount)?;
                        }
                    }
                }
                Opcode::Bang { addr} => {
                    let value = self.pop(addr.clone())?;
                    if let Value::Bool(bool) = value {
                        self.push(addr.clone(), Value::Bool(!bool))?;
                    }
                }
                Opcode::Cond { addr, op} => {
                    let left = self.pop(addr.clone())?;
                    let right = self.pop(addr.clone())?;
                    match op.clone() {
                        _ if op == ">" => self.push(addr.clone(),left.greater(addr.clone(), right)?)?,
                        _ if op == ">=" => self.push(addr.clone(),left.greater_eq(addr.clone(), right)?)?,
                        _ if op == "<" => self.push(addr.clone(),left.less(addr.clone(), right)?)?,
                        _ if op == "<=" => self.push(addr.clone(),left.less_eq(addr.clone(), right)?)?,
                        _ if op == "!=" => self.push(addr.clone(), Value::Bool(left.not_eq(addr.clone(), right)))?,
                        _ if op == "==" => self.push(addr.clone(), Value::Bool(left.eq(addr.clone(), right)))?,
                        _ => {
                            return Err(ControlFlow::Error(Error::new(
                                ErrorType::Runtime,
                                addr.clone(),
                                format!("undefined cond op: {:?}", op),
                                "check your code.".to_string(),
                            )))
                        }
                    }
                }
                Opcode::Logic { addr, op} => {
                    let left = self.pop(addr.clone())?;
                    let right = self.pop(addr.clone())?;
                    match op.clone() {
                        _ if op == "and" => {
                            if let Value::Bool(l) = left {
                                if let Value::Bool(r) = right {
                                    self.push(addr.clone(), Value::Bool(l && r))?
                                } else {
                                    return Err(ControlFlow::Error(Error::new(
                                        ErrorType::Runtime,
                                        addr.clone(),
                                        format!("couldn't use 'and' op with: {:?} and {:?}", left, right),
                                        "check your code.".to_string(),
                                    )))
                                }
                            } else {
                                return Err(ControlFlow::Error(Error::new(
                                    ErrorType::Runtime,
                                    addr.clone(),
                                    format!("couldn't use 'and' op with: {:?} and {:?}", left, right),
                                    "check your code.".to_string(),
                                )))
                            }
                        },
                        _ if op == "or" => {
                            if let Value::Bool(l) = left {
                                if let Value::Bool(r) = right {
                                    self.push(addr.clone(), Value::Bool(l || r))?
                                } else {
                                    return Err(ControlFlow::Error(Error::new(
                                        ErrorType::Runtime,
                                        addr.clone(),
                                        format!("couldn't use 'and' op with: {:?} and {:?}", left, right),
                                        "check your code.".to_string(),
                                    )))
                                }
                            } else {
                                return Err(ControlFlow::Error(Error::new(
                                    ErrorType::Runtime,
                                    addr.clone(),
                                    format!("couldn't use 'and' op with: {:?} and {:?}", left, right),
                                    "check your code.".to_string(),
                                )))
                            }
                        }
                        _ => {
                            return Err(ControlFlow::Error(Error::new(
                                ErrorType::Runtime,
                                addr.clone(),
                                format!("undefined logical op: {:?}", op),
                                "check your code.".to_string(),
                            )))
                        }
                    }
                }
                Opcode::Neg { addr } => {
                    let value = self.pop(addr.clone())?;
                    if let Value::Integer(int) = value {
                        self.push(addr.clone(), Value::Integer(-int))?
                    } else if let Value::Float(float) = value {
                        self.push(addr.clone(), Value::Float(-float))?
                    } else {
                        return Err(ControlFlow::Error(Error::new(
                            ErrorType::Runtime,
                            addr.clone(),
                            format!("couldn't negative value: {:?}", value),
                            "check your code.".to_string(),
                        )))
                    }
                }
                Opcode::Duplicate { addr } => {
                    let value = self.pop(addr.clone())?;
                    self.push(addr.clone(), value.clone())?;
                    self.push(addr.clone(), value.clone())?;
                }
                Opcode::EndLoop { addr, current_iteration } => {
                    return if current_iteration {
                        Err(ControlFlow::Continue)
                    } else {
                        Err(ControlFlow::Break)
                    }
                }
                Opcode::Ret {addr, value} => {
                    self.run(*value.clone(), frame.clone())?;
                    return Err(ControlFlow::Return(self.pop(addr.clone())?))
                }
                Opcode::If {addr, body, cond, elif} => {
                    self.run(*cond.clone(), frame.clone())?;
                    let logical = self.pop(addr.clone())?;
                    if let Value::Bool(bool) = logical {
                        if bool {
                            let mut new_frame = SyncCell::new(Frame::new());
                            let mut new_frame_clone = new_frame.clone();
                            let mut guard = lock!(new_frame);
                            guard.set_root(frame.clone());
                            drop(guard);
                            self.run(*body.clone(), new_frame_clone)?;
                        } else {
                            if let Some(elseif) = elif.clone() {
                                self.run(Chunk::of(*elseif), frame.clone())?;
                            }
                        }
                    }
                }
                Opcode::Loop { addr, body } => {
                    let mut new_frame = SyncCell::new(Frame::new());
                    let mut guard = lock!(new_frame);
                    guard.set_root(frame.clone());
                    drop(guard);
                    loop {
                        if let Err(e) = self.run(*body.clone(), new_frame.clone()) {
                            if let ControlFlow::Continue = e {
                                continue;
                            } else if let ControlFlow::Break = e {
                                break;
                            } else {
                                return Err(e);
                            }
                        }
                    }
                }
                Opcode::Closure { addr, name } => {
                    let mut guard = lock!(frame);
                    let mut closure_object = guard.lookup(addr.clone(), name.clone())?;
                    if let Value::Fn(mut f) = closure_object {
                        let mut fun = lock!(f);
                        fun.closure = Some(frame.clone());
                    }
                }
                Opcode::DefineFn { addr, name, full_name, body, params } => {
                    let mut guard = lock!(frame);
                    let fun = Value::Fn(SyncCell::new(Function::new(
                        name.clone(),
                        *body.clone(),
                        params.clone(),
                    )));
                    guard.define(addr.clone(), name, fun.clone())?;
                    if let Some(f_name) = full_name.clone() {
                        guard.define(addr, f_name, fun)?;
                    }
                }
                Opcode::DefineType { addr, name, full_name, body, constructor } => {
                    let typo = SyncCell::new(Type::new(
                        name.clone(),
                        *body.clone(),
                        constructor
                    ));
                    self.types.insert(name, typo.clone());
                    if let Some(f_name) = full_name.clone() {
                        self.types.insert(f_name, typo);
                    }
                }
                Opcode::DefineUnit { addr, name, full_name, body } => {
                    let unit = Unit::new(
                        self,
                        name.clone(),
                        *body.clone(),
                        frame.clone()
                    )?;
                    // TODO: ROOTING, INIT CALL
                    self.units.insert(name, unit.clone());
                    if let Some(f_name) = full_name.clone() {
                        self.units.insert(f_name, unit);
                    }
                }
                Opcode::Instance { addr, name, args, should_push} => {
                    // args
                    let before = self.eval_stack.len() as i16;
                    self.run(*args.clone(), frame.clone())?;
                    let after = self.eval_stack.len() as i16;
                    let passed_amount = after - before;
                    // instance
                    let typo = self.types.get(&name).clone();
                    if let Some(type_ref) = typo {
                        // instance
                        let instance = Value::Instance(Instance::new(
                            self,
                            type_ref.clone(),
                            addr.clone(),
                            passed_amount,
                            frame.clone()
                        )?);
                        if should_push {
                            self.push(addr.clone(), instance)?;
                        }
                    }
                }
                _ => {
                    println!("undefined opcode: {:?}", op);
                }
            }
        }
        Ok(())
    }

    pub fn call(&mut self, callee: Value, address: Address, root_frame: SyncCell<Frame>, should_push: bool, passed_args: i16) -> Result<(), ControlFlow> {
        match callee {
            Value::Fn(f) => {
                let mut fn_clone = f.clone();
                // клонируем, дабы избежать дедлока.
                let func_guard = lock!(fn_clone);
                let func = func_guard.clone();
                drop(func_guard);
                let mut frame = SyncCell::new(Frame::new());
                let mut frame_guard = lock!(frame);
                frame_guard.set_root(root_frame.clone());
                drop(frame_guard);
                func.run(self, address.clone(), frame.clone(), should_push, passed_args)?;
                Ok(())
            }
            _ => {
                Err(ControlFlow::Error(Error::new(
                    ErrorType::Runtime,
                    address.clone(),
                    format!("couldn't call: {:?}", callee),
                    "check your code.".to_string(),
                )))
            }
        }
    }
}
