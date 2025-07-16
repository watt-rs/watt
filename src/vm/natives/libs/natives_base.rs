// imports
use crate::error;
use crate::errors::errors::Error;
use crate::lexer::address::Address;
use crate::vm::bytecode::OpcodeValue;
use crate::vm::natives::natives;
use crate::vm::table::Table;
use crate::vm::values::Value;
use crate::vm::vm::VM;

/// Provides
#[allow(unused_variables)]
pub unsafe fn provide(built_in_address: &Address, vm: &mut VM) -> Result<(), Error> {
    // panic
    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "base@panic",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            // hint and error texts
            let hint = vm.pop(&addr)?;
            let error = vm.pop(&addr)?;

            if let Value::String(hint_string) = hint {
                if let Value::String(error_string) = error {
                    // raising error, if everything ok
                    error!(Error::own(
                        addr.clone(),
                        (*error_string).clone(),
                        (*hint_string).clone()
                    ));
                } else {
                    error!(Error::own_text(
                        addr.clone(),
                        "error text should be a string.".to_string(),
                        "check your code."
                    ))
                }
            } else {
                error!(Error::own_text(
                    addr.clone(),
                    "hint text should be a string.".to_string(),
                    "check your code."
                ))
            }
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "base@typeof",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let value = vm.pop(&addr)?;
            if !should_push {
                return Ok(());
            }
            match value {
                Value::Float(_) => {
                    vm.op_push(OpcodeValue::String("f64".to_string()), table)?;
                }
                Value::Int(_) => {
                    vm.op_push(OpcodeValue::String("i64".to_string()), table)?;
                }
                Value::String(_) => {
                    vm.op_push(OpcodeValue::String("string".to_string()), table)?;
                }
                Value::Bool(_) => {
                    vm.op_push(OpcodeValue::String("bool".to_string()), table)?;
                }
                Value::Type(_) => {
                    vm.op_push(OpcodeValue::String("type".to_string()), table)?;
                }
                Value::Fn(_) => {
                    vm.op_push(OpcodeValue::String("fn".to_string()), table)?;
                }
                Value::Native(_) => {
                    vm.op_push(OpcodeValue::String("native".to_string()), table)?;
                }
                Value::Instance(i) => {
                    vm.op_push(OpcodeValue::String((*(*i).t).name.name.clone()), table)?;
                }
                Value::Unit(u) => {
                    vm.op_push(OpcodeValue::String((*u).name.name.clone()), table)?;
                }
                Value::Trait(t) => {
                    vm.op_push(OpcodeValue::String((*t).name.name.clone()), table)?;
                }
                Value::List(l) => {
                    vm.op_push(OpcodeValue::String("list".to_string()), table)?;
                }
                Value::Null => {
                    vm.op_push(OpcodeValue::String("null".to_string()), table)?;
                }
                Value::Any(_) => {
                    vm.op_push(OpcodeValue::String("any".to_string()), table)?;
                }
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "base@full_typeof",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let value = vm.pop(&addr)?;
            if !should_push {
                return Ok(());
            }
            match value {
                Value::Float(_) => {
                    vm.op_push(OpcodeValue::String("watt:f64".to_string()), table)?;
                }
                Value::Int(_) => {
                    vm.op_push(OpcodeValue::String("watt:i64".to_string()), table)?;
                }
                Value::String(_) => {
                    vm.op_push(OpcodeValue::String("watt:string".to_string()), table)?;
                }
                Value::Bool(_) => {
                    vm.op_push(OpcodeValue::String("watt:bool".to_string()), table)?;
                }
                Value::Type(_) => {
                    vm.op_push(OpcodeValue::String("watt:type".to_string()), table)?;
                }
                Value::Fn(_) => {
                    vm.op_push(OpcodeValue::String("watt:fn".to_string()), table)?;
                }
                Value::Native(_) => {
                    vm.op_push(OpcodeValue::String("watt:native".to_string()), table)?;
                }
                Value::Instance(i) => {
                    let symbol = (*(*i).t).name.clone();
                    match symbol.full_name {
                        Some(full_name) => {
                            vm.op_push(OpcodeValue::String(full_name), table)?;
                        }
                        None => {
                            vm.op_push(OpcodeValue::String(symbol.name), table)?;
                        }
                    }
                }
                Value::Unit(u) => {
                    let symbol = (*u).name.clone();
                    match symbol.full_name {
                        Some(full_name) => {
                            vm.op_push(OpcodeValue::String(full_name), table)?;
                        }
                        None => {
                            vm.op_push(OpcodeValue::String(symbol.name), table)?;
                        }
                    }
                }
                Value::Trait(t) => {
                    let symbol = (*t).name.clone();
                    match symbol.full_name {
                        Some(full_name) => {
                            vm.op_push(OpcodeValue::String(full_name), table)?;
                        }
                        None => {
                            vm.op_push(OpcodeValue::String(symbol.name), table)?;
                        }
                    }
                }
                Value::List(l) => {
                    vm.op_push(OpcodeValue::String("watt:list".to_string()), table)?;
                }
                Value::Null => {
                    vm.op_push(OpcodeValue::String("watt:null".to_string()), table)?;
                }
                Value::Any(any) => {
                    vm.op_push(OpcodeValue::String("watt:any".to_string()), table)?;
                }
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "base@is_instance",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let value = vm.pop(&addr)?;
            if !should_push {
                return Ok(());
            }
            match value {
                Value::Instance(_) => {
                    vm.op_push(OpcodeValue::Bool(true), table)?;
                }
                _ => {
                    vm.op_push(OpcodeValue::Bool(false), table)?;
                }
            }
            Ok(())
        },
    );
    Ok(())
}
