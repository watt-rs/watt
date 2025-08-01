// imports
use crate::bytecode::OpcodeValue;
use crate::natives::natives;
use crate::table::Table;
use crate::values::Value;
use crate::vm::VM;
use watt_common::address::Address;
use watt_common::{error, errors::Error};

/// Provides
#[allow(unused_variables)]
pub unsafe fn provide(built_in_address: &Address, vm: &mut VM) -> Result<(), Error> {
    // functions
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "convert@to_int",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let value = vm.pop(&addr);

            match value {
                Value::Float(f) => {
                    if should_push {
                        vm.push(Value::Int(f as i64));
                    }
                }
                Value::Int(i) => {
                    if should_push {
                        vm.push(Value::Int(i));
                    }
                }
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
                                format!("could not cast string: {} to i64", *s),
                                "you can convert only number string to i64."
                            ));
                        }
                    }
                }
                Value::Bool(b) => {
                    if b {
                        if should_push {
                            vm.push(Value::Int(1));
                        }
                    } else if should_push {
                        vm.push(Value::Int(0));
                    }
                }
                Value::Null => {
                    if should_push {
                        vm.push(Value::Int(0));
                    }
                }
                _ => {
                    error!(Error::own_text(
                        addr,
                        format!("could not cast value: {value:?} to i64."),
                        "check your value."
                    ));
                }
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "convert@to_float",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let value = vm.pop(&addr);

            match value {
                Value::Float(f) => {
                    if should_push {
                        vm.push(Value::Float(f));
                    }
                }
                Value::Int(i) => {
                    if should_push {
                        vm.push(Value::Float(i as f64));
                    }
                }
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
                                format!("could not cast string: {} to f64", *s),
                                "you can convert only number string to f64."
                            ));
                        }
                    }
                }
                Value::Bool(b) => {
                    if b {
                        if should_push {
                            vm.push(Value::Float(1f64));
                        }
                    } else if should_push {
                        vm.push(Value::Float(0f64));
                    }
                }
                Value::Null => {
                    if should_push {
                        vm.push(Value::Int(0));
                    }
                }
                _ => {
                    error!(Error::own_text(
                        addr,
                        format!("could not cast value: {value:?} to f64"),
                        "check your value"
                    ));
                }
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "convert@to_string",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let value = vm.pop(&addr);

            if should_push {
                vm.op_push(OpcodeValue::String(format!("{value:?}")), table)?;
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "convert@to_bool",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let value = vm.pop(&addr);

            match value {
                Value::Float(f) => {
                    if should_push {
                        if f == 1f64 {
                            vm.push(Value::Bool(true));
                        } else {
                            vm.push(Value::Bool(false));
                        }
                    }
                }
                Value::Int(i) => {
                    if should_push {
                        if i == 1 {
                            vm.push(Value::Bool(true));
                        } else {
                            vm.push(Value::Bool(false));
                        }
                    }
                }
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
                                format!("could not cast string: {} to i64", *s),
                                "you can convert only number string to i64."
                            ));
                        }
                    }
                }
                Value::Bool(b) => {
                    vm.push(Value::Bool(b));
                }
                Value::Null => {
                    if should_push {
                        vm.push(Value::Int(0));
                    }
                }
                _ => {
                    error!(Error::own_text(
                        addr,
                        format!("could not cast value: {value:?} to bool"),
                        "check your value"
                    ));
                }
            }
            Ok(())
        },
    );
    Ok(())
}
