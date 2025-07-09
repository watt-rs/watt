// импорты
use crate::error;
use crate::errors::errors::Error;
use crate::lexer::address::Address;
use crate::vm::bytecode::OpcodeValue;
use crate::vm::natives::natives;
use crate::vm::table::Table;
use crate::vm::values::{Value};
use crate::vm::vm::VM;

// провайд
#[allow(unused_variables)]
pub unsafe fn provide(built_in_address: &Address, vm: &mut VM) -> Result<(), Error> {
    // функции
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "convert@to_int",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let value = vm.pop(&addr)?;
            match value {
                Value::Float(f) => {
                    if should_push { 
                        vm.push(Value::Int(f as i64));
                    }
                },
                Value::Int(i) => {
                    if should_push { 
                        vm.push(Value::Int(i)); 
                    }
                },
                Value::String(s) => {
                    let result = (*s).parse::<i64>();
                    match result {
                        Ok(i) => {
                            if should_push { 
                                vm.push(Value::Int(i));
                            }
                        }
                        Err(_) => {
                            error!(Error::own_text(
                                addr,
                                format!("could not cast string: {} to int", *s),
                                "you can convert only number string to int."
                            ));
                        }
                    }
                },
                Value::Bool(b) => {
                    if b { 
                        if should_push { 
                            vm.push(Value::Int(1));
                        }
                    }
                    else {
                        if should_push {
                            vm.push(Value::Int(0));
                        }
                    }
                },
                Value::Null => {
                    if should_push { 
                        vm.push(Value::Int(0)); 
                    }
                },
                _ => {
                    error!(Error::own_text(
                        addr, 
                        format!("could not cast value: {:?} to int", value), 
                        "check your value"
                    ));
                }
            }
            return Ok(())
        }
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "convert@to_float",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let value = vm.pop(&addr)?;
            match value {
                Value::Float(f) => {
                    if should_push {
                        vm.push(Value::Float(f));
                    }
                },
                Value::Int(i) => {
                    if should_push {
                        vm.push(Value::Float(i as f64));
                    }
                },
                Value::String(s) => {
                    let result = (*s).parse::<f64>();
                    match result {
                        Ok(f) => {
                            if should_push {
                                vm.push(Value::Float(f));
                            }
                        }
                        Err(_) => {
                            error!(Error::own_text(
                                addr,
                                format!("could not cast string: {} to float", *s),
                                "you can convert only number string to float."
                            ));
                        }
                    }
                },
                Value::Bool(b) => {
                    if b {
                        if should_push {
                            vm.push(Value::Float(1f64));
                        }
                    }
                    else {
                        if should_push {
                            vm.push(Value::Float(0f64));
                        }
                    }
                },
                Value::Null => {
                    if should_push {
                        vm.push(Value::Int(0));
                    }
                },
                _ => {
                    error!(Error::own_text(
                        addr, 
                        format!("could not cast value: {:?} to float", value), 
                        "check your value"
                    ));
                }
            }
            return Ok(())
        }
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "convert@to_string",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let value = vm.pop(&addr)?;
            if should_push {
                vm.op_push(
                    OpcodeValue::String(
                        format!("{:?}", value)
                    ),
                    table
                )?;
            }
            Ok(())
        }
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "convert@to_bool",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let value = vm.pop(&addr)?;
            match value {
                Value::Float(f) => {
                    if should_push {
                        if f == 1f64 {
                            vm.push(Value::Bool(true));
                        }
                        else {
                            vm.push(Value::Bool(false));
                        }
                    }
                },
                Value::Int(i) => {
                    if should_push {
                        if i == 1 {
                            vm.push(Value::Bool(true));
                        }
                        else {
                            vm.push(Value::Bool(false));
                        }
                    }
                },
                Value::String(s) => {
                    let result = (*s).parse::<bool>();
                    match result {
                        Ok(b) => {
                            if should_push {
                                vm.push(Value::Bool(b));
                            }
                        }
                        Err(_) => {
                            error!(Error::own_text(
                                addr,
                                format!("could not cast string: {} to int", *s),
                                "you can convert only number string to int."
                            ));
                        }
                    }
                },
                Value::Bool(b) => {
                    vm.push(Value::Bool(b));
                },
                Value::Null => {
                    if should_push {
                        vm.push(Value::Int(0));
                    }
                },
                _ => {
                    error!(Error::own_text(
                        addr, 
                        format!("could not cast value: {:?} to bool", value), 
                        "check your value"
                    ));
                }
            }
            Ok(())
        }
    );    
    // успех
    Ok(())
}