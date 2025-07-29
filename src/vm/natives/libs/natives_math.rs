// imports
use crate::error;
use crate::errors::errors::Error;
use crate::lexer::address::Address;
use crate::vm::natives::natives;
use crate::vm::table::Table;
use crate::vm::values::Value;
use crate::vm::vm::VM;

/// Provides
#[allow(unused_variables)]
pub unsafe fn provide(built_in_address: &Address, vm: &mut VM) -> Result<(), Error> {
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "math@sin",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let value = vm.pop(&addr)?;

            if should_push {
                match value {
                    Value::Float(f64) => vm.push(Value::Float(f64::sin(f64))),
                    Value::Int(i64) => vm.push(Value::Float(f64::sin(i64 as f64))),
                    _ => {
                        error!(Error::own_text(
                            addr,
                            format!("could not use sin with {value}"),
                            "you can use i64 or f64."
                        ));
                    }
                }
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "math@cos",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let value = vm.pop(&addr)?;

            if should_push {
                match value {
                    Value::Float(f64) => vm.push(Value::Float(f64::cos(f64))),
                    Value::Int(i64) => vm.push(Value::Float(f64::cos(i64 as f64))),
                    _ => {
                        error!(Error::own_text(
                            addr,
                            format!("could not use cos with {value}"),
                            "you can use i64 or f64."
                        ));
                    }
                }
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "math@asin",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let value = vm.pop(&addr)?;

            if should_push {
                match value {
                    Value::Float(f64) => vm.push(Value::Float(f64::asin(f64))),
                    Value::Int(i64) => vm.push(Value::Float(f64::asin(i64 as f64))),
                    _ => {
                        error!(Error::own_text(
                            addr,
                            format!("could not use asin with {value}"),
                            "you can use i64 or f64."
                        ));
                    }
                }
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "math@acos",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let value = vm.pop(&addr)?;

            if should_push {
                match value {
                    Value::Float(f64) => vm.push(Value::Float(f64::acos(f64))),
                    Value::Int(i64) => vm.push(Value::Float(f64::acos(i64 as f64))),
                    _ => {
                        error!(Error::own_text(
                            addr,
                            format!("could not use acos with {value}"),
                            "you can use i64 or f64."
                        ));
                    }
                }
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "math@atan",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let value = vm.pop(&addr)?;

            if should_push {
                match value {
                    Value::Float(f64) => vm.push(Value::Float(f64::atan(f64))),
                    Value::Int(i64) => vm.push(Value::Float(f64::atan(i64 as f64))),
                    _ => {
                        error!(Error::own_text(
                            addr,
                            format!("could not use atan with {value}"),
                            "you can use i64 or f64."
                        ));
                    }
                }
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "math@tan",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let value = vm.pop(&addr)?;

            if should_push {
                match value {
                    Value::Float(f64) => vm.push(Value::Float(f64::tan(f64))),
                    Value::Int(i64) => vm.push(Value::Float(f64::tan(i64 as f64))),
                    _ => {
                        error!(Error::own_text(
                            addr,
                            format!("could not use tan with {value}"),
                            "you can use i64 or f64."
                        ));
                    }
                }
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "math@ctg",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let value = vm.pop(&addr)?;

            if should_push {
                match value {
                    Value::Float(f64) => vm.push(Value::Float(1.0 / f64::tan(f64))),
                    Value::Int(i64) => vm.push(Value::Float(1.0 / f64::tan(i64 as f64))),
                    _ => {
                        error!(Error::own_text(
                            addr,
                            format!("could not use cat with {value}"),
                            "you can use i64 or f64."
                        ));
                    }
                }
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "math@tanh",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let value = vm.pop(&addr)?;

            if should_push {
                match value {
                    Value::Float(f64) => vm.push(Value::Float(f64::tanh(f64))),
                    Value::Int(i64) => vm.push(Value::Float(f64::tanh(i64 as f64))),
                    _ => {
                        error!(Error::own_text(
                            addr,
                            format!("could not use tanh with {value}"),
                            "you can use i64 or f64."
                        ));
                    }
                }
            }

            Ok(())
        },
    );
    Ok(())
}
