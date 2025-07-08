// импорты
use crate::error;
use crate::errors::errors::Error;
use crate::lexer::address::Address;
use crate::vm::bytecode::OpcodeValue;
use crate::vm::memory::memory;
use crate::vm::natives::natives;
use crate::vm::table::Table;
use crate::vm::values::Value;
use crate::vm::vm::VM;

use std::io::{Read, Write};

pub unsafe fn provide(built_in_address: Address, vm: &mut VM) -> Result<(), Error> {
    // функции
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "fs@open".to_string(),
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let filename = match vm.pop(&addr) {
                Ok(Value::String(filename)) => &*filename,
                Ok(a) => {
                    error!(Error::own_text(
                        addr.clone(),
                        format!("Expected string, found {:?}", a),
                        "check your code"
                    ));
                }
                Err(_) => {
                    todo!()
                }
            };

            // если надо пушить
            if should_push {
                let file = match std::fs::OpenOptions::new().read(true).open(&filename) {
                    Ok(file) => file,
                    Err(e) => {
                        vm.op_push(OpcodeValue::Raw(Value::Null), table)?;

                        return Ok(());
                    }
                };

                let file = memory::alloc_value(file);

                // добавляем
                vm.op_push(OpcodeValue::Raw(Value::Any(file)), table)?;
            }
            // успех
            Ok(())
        },
    );

    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "fs@create".to_string(),
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let filename = match vm.pop(&addr) {
                Ok(Value::String(filename)) => &*filename,
                Ok(a) => {
                    error!(Error::own_text(
                        addr.clone(),
                        format!("Expected string, found {:?}", a),
                        "check your code"
                    ));
                }
                Err(_) => {
                    todo!()
                }
            };

            // если надо пушить
            if should_push {
                let file = match std::fs::OpenOptions::new().read(true).write(true).create(true).open(&filename) {
                    Ok(file) => file,
                    Err(_e) => {
                        vm.op_push(OpcodeValue::Raw(Value::Null), table)?;

                        return Ok(());
                    }
                };

                let file = memory::alloc_value(file);

                // добавляем
                vm.op_push(OpcodeValue::Raw(Value::Any(file)), table)?;
            }
            // успех
            Ok(())
        },
    );

    pub fn get_instance<'vm>(vm: &'vm mut VM, addr: &Address) -> &'vm mut std::fs::File {
        match vm.pop(&addr) {
            Ok(Value::Any(instance)) => {
                let instance = unsafe { &mut *instance };

                if !instance.is::<std::fs::File>() {
                    error!(Error::own_text(
                        addr.clone(),
                        format!("Internal type in std.fs.File is not a Rust's `std::io::File`!"),
                        "please file an issue at https://github.com/vyacheslavhere/watt"
                    ));
                }

                instance.downcast_mut().unwrap()
            }
            Ok(a) => {
                error!(Error::own_text(
                    addr.clone(),
                    format!("Expected instance, found {:?}", a),
                    "check your code"
                ));
            }
            Err(_) => {
                todo!()
            }
        }
    }

    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "fs@read_to_string".to_string(),
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let instance: &mut std::fs::File = get_instance(vm, &addr);

            // если надо пушить
            if should_push {
                let mut string = String::new();
                instance.read_to_string(&mut string).unwrap();

                vm.op_push(
                    OpcodeValue::Raw(Value::String(memory::alloc_value(string))),
                    table,
                )?;
            }
            // успех
            Ok(())
        },
    );

    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "fs@write".to_string(),
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let data = match vm.pop(&addr) {
                Ok(Value::String(string)) => {
                    unsafe { &*string }
                }
                Ok(a) => {
                    error!(Error::own_text(
                        addr.clone(),
                        format!("Expected string, found {:?}", a),
                        "check your code"
                    ));
                }
                Err(_) => {
                    todo!()
                }
            };
            
            let instance: &mut std::fs::File = get_instance(vm, &addr);

            // если надо пушить
            if should_push {
                let value = instance.write(data.as_bytes());

                match value {
                    Ok(_) => {
                        vm.op_push(                    
                            OpcodeValue::Raw(Value::Null),
                            table,
                        )?;
                    }
                    Err(e) => {
                        vm.op_push(                    
                            OpcodeValue::Raw(Value::Int(e.raw_os_error().unwrap_or(0) as _)),
                            table,
                        )?;
                    }
                }
            }
            // успех
            Ok(())
        },
    );

    // TODO: Write, Close

    // успех
    Ok(())
}
