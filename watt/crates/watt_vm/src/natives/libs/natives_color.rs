use colored::Colorize;
use watt_common::{address::Address, error, errors::Error};

use crate::bytecode::OpcodeValue;
use crate::natives::natives;
use crate::table::Table;
use crate::values::Value;
use crate::vm::VM;

pub unsafe fn provide(built_in_address: &Address, vm: &mut VM) -> Result<(), Error> {
    // functions

    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "color@paint_red",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let value = vm.pop(&addr);

            fn paint(str: String) -> String {
                str.red().input
            }

            match value {
                Value::String(string) => {
                    if should_push {
                        vm.op_push(OpcodeValue::String(paint((*string).clone())), table)?;
                    }
                }

                Value::Float(f) => {
                    if should_push {
                        vm.op_push(OpcodeValue::String(paint(f.to_string())), table)?;
                    }
                }

                Value::Int(i) => {
                    if should_push {
                        vm.op_push(OpcodeValue::String(paint(i.to_string())), table)?;
                    }
                }

                Value::Bool(b) => {
                    if should_push {
                        vm.op_push(OpcodeValue::String(paint(b.to_string())), table)?;
                    }
                }

                Value::Null => {
                    if should_push {
                        vm.op_push(OpcodeValue::String(paint("null".to_string())), table)?;
                    }
                }

                _ => {
                    error!(Error::own_text(
                        addr,
                        format!("could not cast string: {} to string", value),
                        "check your code",
                    ))
                }
            }

            Ok(())
        },
    );

    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "color@paint_blue",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let value = vm.pop(&addr);

            fn paint(str: String) -> String {
                str.blue().input
            }

            match value {
                Value::String(string) => {
                    if should_push {
                        vm.op_push(OpcodeValue::String(paint((*string).clone())), table)?;
                    }
                }

                Value::Float(f) => {
                    if should_push {
                        vm.op_push(OpcodeValue::String(paint(f.to_string())), table)?;
                    }
                }

                Value::Int(i) => {
                    if should_push {
                        vm.op_push(OpcodeValue::String(paint(i.to_string())), table)?;
                    }
                }

                Value::Bool(b) => {
                    if should_push {
                        vm.op_push(OpcodeValue::String(paint(b.to_string())), table)?;
                    }
                }

                Value::Null => {
                    if should_push {
                        vm.op_push(OpcodeValue::String(paint("null".to_string())), table)?;
                    }
                }

                _ => {
                    error!(Error::own_text(
                        addr,
                        format!("could not cast string: {} to string", value),
                        "check your code",
                    ))
                }
            }

            Ok(())
        },
    );

    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "color@paint_cyan",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let value = vm.pop(&addr);

            fn paint(str: String) -> String {
                str.cyan().input
            }

            match value {
                Value::String(string) => {
                    if should_push {
                        vm.op_push(OpcodeValue::String(paint((*string).clone())), table)?;
                    }
                }

                Value::Float(f) => {
                    if should_push {
                        vm.op_push(OpcodeValue::String(paint(f.to_string())), table)?;
                    }
                }

                Value::Int(i) => {
                    if should_push {
                        vm.op_push(OpcodeValue::String(paint(i.to_string())), table)?;
                    }
                }

                Value::Bool(b) => {
                    if should_push {
                        vm.op_push(OpcodeValue::String(paint(b.to_string())), table)?;
                    }
                }

                Value::Null => {
                    if should_push {
                        vm.op_push(OpcodeValue::String(paint("null".to_string())), table)?;
                    }
                }

                _ => {
                    error!(Error::own_text(
                        addr,
                        format!("could not cast string: {} to string", value),
                        "check your code",
                    ))
                }
            }

            Ok(())
        },
    );

    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "color@paint_yellow",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let value = vm.pop(&addr);

            fn paint(str: String) -> String {
                str.yellow().input
            }

            match value {
                Value::String(string) => {
                    if should_push {
                        vm.op_push(OpcodeValue::String(paint((*string).clone())), table)?;
                    }
                }

                Value::Float(f) => {
                    if should_push {
                        vm.op_push(OpcodeValue::String(paint(f.to_string())), table)?;
                    }
                }

                Value::Int(i) => {
                    if should_push {
                        vm.op_push(OpcodeValue::String(paint(i.to_string())), table)?;
                    }
                }

                Value::Bool(b) => {
                    if should_push {
                        vm.op_push(OpcodeValue::String(paint(b.to_string())), table)?;
                    }
                }

                Value::Null => {
                    if should_push {
                        vm.op_push(OpcodeValue::String(paint("null".to_string())), table)?;
                    }
                }

                _ => {
                    error!(Error::own_text(
                        addr,
                        format!("could not cast string: {} to string", value),
                        "check your code",
                    ))
                }
            }

            Ok(())
        },
    );

    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "color@paint_green",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let value = vm.pop(&addr);

            fn paint(str: String) -> String {
                str.green().input
            }

            match value {
                Value::String(string) => {
                    if should_push {
                        vm.op_push(OpcodeValue::String(paint((*string).clone())), table)?;
                    }
                }

                Value::Float(f) => {
                    if should_push {
                        vm.op_push(OpcodeValue::String(paint(f.to_string())), table)?;
                    }
                }

                Value::Int(i) => {
                    if should_push {
                        vm.op_push(OpcodeValue::String(paint(i.to_string())), table)?;
                    }
                }

                Value::Bool(b) => {
                    if should_push {
                        vm.op_push(OpcodeValue::String(paint(b.to_string())), table)?;
                    }
                }

                Value::Null => {
                    if should_push {
                        vm.op_push(OpcodeValue::String(paint("null".to_string())), table)?;
                    }
                }

                _ => {
                    error!(Error::own_text(
                        addr,
                        format!("could not cast string: {} to string", value),
                        "check your code",
                    ))
                }
            }

            Ok(())
        },
    );

    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "color@paint_magenta",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let value = vm.pop(&addr);

            fn paint(str: String) -> String {
                str.magenta().input
            }

            match value {
                Value::String(string) => {
                    if should_push {
                        vm.op_push(OpcodeValue::String(paint((*string).clone())), table)?;
                    }
                }

                Value::Float(f) => {
                    if should_push {
                        vm.op_push(OpcodeValue::String(paint(f.to_string())), table)?;
                    }
                }

                Value::Int(i) => {
                    if should_push {
                        vm.op_push(OpcodeValue::String(paint(i.to_string())), table)?;
                    }
                }

                Value::Bool(b) => {
                    if should_push {
                        vm.op_push(OpcodeValue::String(paint(b.to_string())), table)?;
                    }
                }

                Value::Null => {
                    if should_push {
                        vm.op_push(OpcodeValue::String(paint("null".to_string())), table)?;
                    }
                }

                _ => {
                    error!(Error::own_text(
                        addr,
                        format!("could not cast string: {} to string", value),
                        "check your code",
                    ))
                }
            }

            Ok(())
        },
    );

    Ok(())
}
