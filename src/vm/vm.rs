use crate::errors::*;
use crate::lexer::address::Address;
use crate::vm::bytecode::*;
use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex};
use crate::vm::frames::Frame;
use crate::vm::values::{Native, Value};
use super::bytecode::Opcode;


// vm struct
pub struct Vm {
    eval_stack: VecDeque<Value>,
    natives: BTreeMap<String, Native>
}

// control flow
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
                ("println".to_string(), |vm: &mut Vm, address: Address, should_push: bool| -> Result<(), ControlFlow> {
                    let value = vm.pop(address)?;
                    println!("{:?}", value);
                    return Ok(());
                } as Native)
            ])
        }
    }

    // push to stack
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

    pub fn run(&mut self, chunk: Chunk, frame: Arc<Mutex<Frame>>) -> Result<(), ControlFlow> {
        for op in chunk.opcodes() {
            match op {
                // push
                Opcode::Push {
                    addr: address,
                    value,
                } => {
                    self.push(address.clone(), value)?;
                }
                // pop
                Opcode::Pop { addr: address } => {
                    self.pop(address.clone())?;
                }
                // binary
                Opcode::Bin { addr: address, op } => {
                    let left = self.pop(address.clone())?;
                    let right = self.pop(address.clone())?;

                    match left {
                        Value::Integer(l) => {
                            match right {
                                Value::Integer(r) => {
                                    self.push(address, Value::Integer(l + r))?;
                                }
                                Value::Float(r) => {
                                    self.push(address, Value::Float(l as f64 + r))?;
                                }
                                _ => {
                                    return Err(ControlFlow::Error(Error::new(
                                        ErrorType::Runtime,
                                        address,
                                        format!(
                                            "couldn't add number with: {:?}",
                                            right
                                        ),
                                        "check your code.".to_string(),
                                    )));
                                }
                            }
                        }
                        Value::Float(l) => {
                            match right {
                                Value::Integer(r) => {
                                    self.push(address, Value::Float(l + r as f64))?;
                                }
                                Value::Float(r) => {
                                    self.push(address, Value::Float(l + r))?;
                                }
                                _ => {
                                    return Err(ControlFlow::Error(Error::new(
                                        ErrorType::Runtime,
                                        address,
                                        format!(
                                            "couldn't add number with: {:?}",
                                            right
                                        ),
                                        "check your code.".to_string(),
                                    )));
                                }
                            }
                        }
                        Value::String(l) => {
                            match right {
                                Value::String(r) => {
                                    self.push(address, Value::String(
                                        format!("{:?}{:?}", l, r)
                                    ))?;
                                }
                                _ => {
                                    return Err(ControlFlow::Error(Error::new(
                                        ErrorType::Runtime,
                                        address,
                                        format!("couldn't add string with: {:?}", right),
                                        "check your code.".to_string(),
                                    )));
                                }
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
                                let instance_lock = instance.lock().unwrap();
                                let fields_lock = instance_lock.fields.lock().unwrap();
                                if should_push {
                                    self.push(address.clone(), fields_lock.lookup(address.clone(), name.clone())?)?
                                }
                            }
                            Value::Unit(unit) => {
                                let unit_lock = unit.lock().unwrap();
                                let fields_lock = unit_lock.fields.lock().unwrap();
                                if should_push {
                                    self.push(address.clone(), fields_lock.lookup(address.clone(), name.clone())?)?
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
                            let frame_lock = frame.lock().unwrap();
                            self.push(address.clone(), frame_lock.lookup(address.clone(), name.clone())?)?;
                        }
                    }
                }
                // define
                Opcode::Define { addr: address, name, has_previous, value } => {
                    if has_previous {
                        let previous = self.pop(address.clone())?.clone();
                        match previous {
                            Value::Instance(instance) => {
                                let instance_lock = instance.lock().unwrap();
                                let mut fields_lock = instance_lock.fields.lock().unwrap();
                                self.run(*value, frame.clone())?;
                                fields_lock.define(address.clone(), name.clone(), self.pop(address.clone())?)?;
                            }
                            Value::Unit(unit) => {
                                let unit_lock = unit.lock().unwrap();
                                let mut fields_lock = unit_lock.fields.lock().unwrap();
                                self.run(*value, frame.clone())?;
                                fields_lock.define(address.clone(), name.clone(), self.pop(address.clone())?)?;
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
                        self.run(*value, frame.clone())?;
                        let mut frame_lock = frame.lock().unwrap();
                        frame_lock.define(address.clone(), name.clone(), self.pop(address.clone())?)?;
                    }
                }
                // set
                Opcode::Set { addr: address, name, has_previous, value } => {
                    if has_previous {
                        let previous = self.pop(address.clone())?.clone();
                        match previous {
                            Value::Instance(instance) => {
                                self.run(*value, frame.clone())?;
                                let instance_lock = instance.lock().unwrap();
                                let mut fields_lock = instance_lock.fields.lock().unwrap();
                                fields_lock.set(address.clone(), name.clone(), self.pop(address.clone())?)?;
                            }
                            Value::Unit(unit) => {
                                self.run(*value, frame.clone())?;
                                let unit_lock = unit.lock().unwrap();
                                let mut fields_lock = unit_lock.fields.lock().unwrap();
                                fields_lock.set(address.clone(), name.clone(), self.pop(address.clone())?)?;
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
                        self.run(*value, frame.clone())?;
                        let mut frame_lock = frame.lock().unwrap();
                        frame_lock.set(address.clone(), name.clone(), self.pop(address.clone())?)?;
                    }
                }
                // call
                Opcode::Call { addr, name, args, has_previous, should_push} => {
                    // args
                    let before = self.eval_stack.len() as i64;
                    self.run(*args, frame.clone())?;
                    let after = self.eval_stack.len() as i64;
                    let passed_amount = after - before;
                    // call
                    if has_previous {
                        let previous = self.pop(addr.clone())?.clone();
                        match previous {
                            Value::Instance(instance) => {
                                let instance_lock = instance.lock().unwrap();
                                let mut fields_lock = instance_lock.fields.lock().unwrap();
                                let callee = fields_lock.lookup(addr.clone(), name.clone())?;
                                self.call(callee, addr, frame.clone(), should_push)?;
                            }
                            Value::Unit(unit) => {
                                let unit_lock = unit.lock().unwrap();
                                let mut fields_lock = unit_lock.fields.lock().unwrap();
                                let callee = fields_lock.lookup(addr.clone(), name.clone())?;
                                self.call(callee, addr, frame.clone(), should_push)?;
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
                        let native = self.natives.get(&name);
                        if let Some(native_ref) = native {
                            // native
                            native_ref(self, addr.clone(), should_push)?;
                        } else {
                            // frame fn
                            let frame_lock = frame.lock().unwrap();
                            self.call(frame_lock.lookup(addr.clone(), name.clone())?, addr.clone(), frame.clone(), should_push)?;
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

    pub fn call(&mut self, callee: Value, address: Address, frame: Arc<Mutex<Frame>>, should_push: bool) -> Result<(), ControlFlow> {
        match callee {
            Value::Fn(f) => {
                let mut fun = f.lock().unwrap();
                fun.run(self, address.clone(), frame.clone(), should_push)?;
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
