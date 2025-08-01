// imports
use crate::bytecode::OpcodeValue;
use crate::flow::ControlFlow;
use crate::memory::memory;
use crate::natives::natives;
use crate::natives::utils;
use crate::table::Table;
use crate::values::Value;
use crate::vm::VM;
use std::io::{Read, Seek, Write};
use watt_common::address::Address;
use watt_common::{error, errors::Error};

/// Gets file from stack
unsafe fn pop_file<'vm>(
    vm: &'vm mut VM,
    addr: &Address,
) -> Result<&'vm mut std::fs::File, ControlFlow> {
    // getting a raw file
    let raw_file = utils::expect_any(addr, vm.pop(addr), None);

    if !(*raw_file).is::<std::fs::File>() {
        error!(Error::new(
            addr.clone(),
            "internal type in std.fs.File is not a Rust's `std::io::File`!",
            "please, file an issue at https://github.com/vyacheslavhere/watt"
        ));
    }

    Ok((*raw_file).downcast_mut().unwrap())
}

/// Provides
pub unsafe fn provide(built_in_address: &Address, vm: &mut VM) -> Result<(), Error> {
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "fs@open",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            // file name
            let filename = utils::expect_cloned_string(&addr, vm.pop(&addr));

            if should_push {
                // opening file for reading
                let file = match std::fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(filename)
                {
                    Ok(file) => file,
                    Err(_) => {
                        vm.op_push(OpcodeValue::Raw(Value::Null), table)?;
                        return Ok(());
                    }
                };
                let file = memory::alloc_value(file);
                vm.op_push(OpcodeValue::Raw(Value::Any(file)), table)?;
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "fs@create",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            // file name
            let filename = utils::expect_cloned_string(&addr, vm.pop(&addr));

            if should_push {
                // opening file for reading, writing, creating
                let file = match std::fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(false)
                    .open(filename)
                {
                    Ok(file) => file,
                    Err(_e) => {
                        vm.op_push(OpcodeValue::Raw(Value::Null), table)?;

                        return Ok(());
                    }
                };
                let file = memory::alloc_value(file);
                vm.op_push(OpcodeValue::Raw(Value::Any(file)), table)?;
            }

            Ok(())
        },
    );
    // continue providing
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "fs@read_to_string",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            // getting raw file
            let file: &mut std::fs::File = pop_file(vm, &addr)?;

            // reading and pushing text to string
            if should_push {
                let mut string = String::new();
                file.read_to_string(&mut string).unwrap();
                vm.op_push(OpcodeValue::String(string), table)?;
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "fs@write",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            // getting data for writing
            let data = utils::expect_cloned_string(&addr, vm.pop(&addr));
            // getting raw file
            let file: &mut std::fs::File = pop_file(vm, &addr)?;

            // writing
            if should_push {
                let value = file.write(data.as_bytes());
                match value {
                    Ok(_) => {
                        vm.op_push(OpcodeValue::Raw(Value::Null), table)?;
                    }
                    Err(e) => {
                        vm.op_push(OpcodeValue::Int(e.raw_os_error().unwrap_or(0) as _), table)?;
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
        "fs@tell",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            // getting raw file
            let file: &mut std::fs::File = pop_file(vm, &addr)?;

            if should_push {
                let value = file.stream_position().unwrap_or(0);
                vm.op_push(OpcodeValue::Int(value as _), table)?;
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        3,
        "fs@seek",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            // getting whence and position
            let whence = utils::expect_int(&addr, vm.pop(&addr));
            let position = utils::expect_int(&addr, vm.pop(&addr));
            // getting raw file
            let file: &mut std::fs::File = pop_file(vm, &addr)?;

            // performing seek
            file.seek(match whence {
                1 => std::io::SeekFrom::Current(position as _),
                2 => std::io::SeekFrom::End(position as _),
                _ => std::io::SeekFrom::Start(position as _),
            })
            .unwrap();
            if should_push {
                vm.op_push(OpcodeValue::Raw(Value::Null), table)?;
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "fs@mkdir",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            // getting directory name
            let name = utils::expect_cloned_string(&addr, vm.pop(&addr));

            // creating directory
            let result = std::fs::create_dir(name);
            if should_push {
                if let Err(e) = result {
                    vm.op_push(OpcodeValue::Int(e.raw_os_error().unwrap_or(0) as _), table)?;
                } else {
                    vm.op_push(OpcodeValue::Raw(Value::Null), table)?;
                }
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "fs@delete_directory",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            // getting directory name
            let name = utils::expect_cloned_string(&addr, vm.pop(&addr));

            // deleting directory
            let result = std::fs::remove_dir(name);
            if should_push {
                if let Err(e) = result {
                    vm.op_push(OpcodeValue::Int(e.raw_os_error().unwrap_or(0) as _), table)?;
                } else {
                    vm.op_push(OpcodeValue::Raw(Value::Null), table)?;
                }
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "fs@delete_directory_all",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            // getting directory name
            let name = utils::expect_cloned_string(&addr, vm.pop(&addr));

            // deleting directory tree
            let result = std::fs::remove_dir_all(name);
            if should_push {
                if let Err(e) = result {
                    vm.op_push(OpcodeValue::Int(e.raw_os_error().unwrap_or(0) as _), table)?;
                } else {
                    vm.op_push(OpcodeValue::Raw(Value::Null), table)?;
                }
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "fs@exists",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            // getting path and checking existence
            let path = utils::expect_cloned_string(&addr, vm.pop(&addr));
            let result = std::fs::exists(path);

            if should_push {
                if let Ok(data) = result {
                    vm.op_push(OpcodeValue::Raw(Value::Bool(data)), table)?;
                } else {
                    // NDRAEY todo: Change it when I learn to use typeof (return errno)
                    vm.op_push(OpcodeValue::Raw(Value::Null), table)?;
                }
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "fs@list",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            // getting path
            let path = utils::expect_cloned_string(&addr, vm.pop(&addr));

            // reading directory
            if should_push {
                let result = std::fs::read_dir(path);
                match result {
                    Err(_) => {
                        // NDRAEY todo: Change it when I learn to use typeof (return errno)
                        vm.op_push(OpcodeValue::Raw(Value::Null), table)?;
                    }
                    Ok(data) => {
                        let paths: Vec<Value> = data
                            .filter(Result::is_ok)
                            .map(|x| x.unwrap().path().to_string_lossy().to_string())
                            .map(|st| Value::String(memory::alloc_value(st)))
                            .collect();
                        let allocated = Value::List(memory::alloc_value(paths));
                        vm.op_push(OpcodeValue::Raw(allocated), table)?;
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
        "fs@is_directory",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            // getting path
            let path = utils::expect_cloned_string(&addr, vm.pop(&addr));

            if should_push {
                let result = std::fs::metadata(path);
                match result {
                    Err(_) => {
                        vm.op_push(OpcodeValue::Raw(Value::Null), table)?;
                    }
                    Ok(data) => {
                        vm.op_push(OpcodeValue::Bool(data.is_dir()), table)?;
                    }
                }
            }

            Ok(())
        },
    );
    Ok(())
}
