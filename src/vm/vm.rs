use crate::errors::*;
use crate::lexer::address::Address;
use crate::vm::bytecode::*;
use std::collections::VecDeque;
use crate::vm::values::Value;
use super::bytecode::Opcode;


// vm struct
pub struct Vm {
    eval_stack: VecDeque<Value>,
}

// vm impl
impl Vm {
    // new vm instance
    pub fn new() -> Self {
        Vm {
            eval_stack: VecDeque::new(),
        }
    }

    // push to stack
    pub fn push(&mut self, address: Address, value: Value) -> Result<(), Error> {
        self.eval_stack.push_back(value);
        Ok(())
    }

    pub fn pop(&mut self, address: Address) -> Result<Value, Error> {
        match self.eval_stack.pop_back() {
            Some(v) => Ok(v),
            None => Err(Error::new(
                ErrorType::Runtime,
                address,
                "stack is empty.".to_string(),
                "check your code.".to_string(),
            )),
        }
    }

    pub fn run(&mut self, chunk: Chunk) -> Result<(), Error> {
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
                                    return Err(Error::new(
                                        ErrorType::Runtime,
                                        address,
                                        format!(
                                            "couldn't add number with: {:?}",
                                            right
                                        ),
                                        "check your code.".to_string(),
                                    ));
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
                                    return Err(Error::new(
                                        ErrorType::Runtime,
                                        address,
                                        format!(
                                            "couldn't add number with: {:?}",
                                            right
                                        ),
                                        "check your code.".to_string(),
                                    ));
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
                                    return Err(Error::new(
                                        ErrorType::Runtime,
                                        address,
                                        format!("couldn't add string with: {:?}", right),
                                        "check your code.".to_string(),
                                    ));
                                }
                            }
                        }
                        _ => {}
                    }

                }
                _ => {}
            }
        }
        Ok(())
    }
}
