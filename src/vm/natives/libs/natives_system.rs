// imports
use crate::error;
use crate::errors::errors::Error;
use crate::lexer::address::Address;
use crate::vm::bytecode::OpcodeValue;
use crate::vm::memory::memory::{self};
use crate::vm::natives::natives;
use crate::vm::natives::utils;
use crate::vm::table::Table;
use crate::vm::values::Value;
use crate::vm::vm::VM;
use std::process::Command;
use sysinfo::System;

/// Provides
#[allow(unused_variables)]
pub unsafe fn provide(built_in_address: &Address, vm: &mut VM) -> Result<(), Error> {
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "system@getenv",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            if !should_push {
                return Ok(());
            }

            let env_key = &*utils::expect_string(&addr, vm.pop(&addr));
            let value = match std::env::vars().find(|x| &x.0 == env_key) {
                Some((key, value)) => value,
                None => {
                    vm.push(Value::Null);
                    return Ok(());
                }
            };

            vm.op_push(OpcodeValue::String(value), table)?;

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "system@setenv",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let env_value = &*utils::expect_string(&addr, vm.pop(&addr));
            let env_key = &*utils::expect_string(&addr, vm.pop(&addr));

            std::env::set_var(env_key, env_value);

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        0,
        "system@getcwd",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            if !should_push {
                return Ok(());
            }

            // getting cwd with error handling
            let cwd = match std::env::current_dir() {
                Ok(cwd) => {
                    let path = cwd.to_str().map(|x| x.to_string());
                    match path {
                        Some(p) => p,
                        None => {
                            vm.push(Value::Null);
                            return Ok(());
                        }
                    }
                }
                Err(e) => {
                    vm.push(Value::Null);
                    return Ok(());
                }
            };

            vm.op_push(OpcodeValue::String(cwd), table)?;
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        0,
        "system@getargs",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            if !should_push {
                return Ok(());
            }

            // parsing args
            let args: Vec<Value> = std::env::args()
                .skip(1)
                .map(|x| {
                    let string = Value::String(memory::alloc_value(x));

                    vm.gc_guard(string);
                    vm.gc_register(string, table);

                    string
                })
                .collect();

            // unguarding strings
            for _ in 0..args.len() {
                vm.gc_unguard();
            }

            // safety of strings will not be erased
            // guaranteed by list marking if gc will invoke.
            let raw_list = Value::List(memory::alloc_value(args));
            vm.op_push(OpcodeValue::Raw(raw_list), table)?;

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        0,
        "system@memory_total",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            if !should_push {
                return Ok(());
            }

            let system_info = System::new_with_specifics(
                sysinfo::RefreshKind::nothing()
                    .with_memory(sysinfo::MemoryRefreshKind::everything()),
            );
            vm.push(Value::Int(system_info.total_memory() as _));

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        0,
        "system@memory_used",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            if !should_push {
                return Ok(());
            }

            let system_info = System::new_with_specifics(
                sysinfo::RefreshKind::nothing()
                    .with_memory(sysinfo::MemoryRefreshKind::everything()),
            );
            vm.push(Value::Int(system_info.used_memory() as _));

            Ok(())
        },
    );

    natives::provide(
        vm,
        built_in_address.clone(),
        0,
        "system@cpu_count",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            if !should_push {
                return Ok(());
            }

            vm.push(Value::Int(
                std::thread::available_parallelism()
                    .map(|x| x.get())
                    .unwrap_or(1) as _,
            ));

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        0,
        "system@this_process_id",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            vm.push(Value::Int(std::process::id() as _));
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "system@this_process_terminate",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let code = utils::expect_int(&addr, vm.pop(&addr));
            std::process::exit(code as _);
        },
    );

    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "system@process_spawn_shell",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            if !should_push {
                error!(Error::new(
                    addr.clone(),
                    "A value must be taken.",
                    "Give it a name: `process = std.process.spawn(...)`"
                ));
            }

            let command = &*utils::expect_string(&addr, vm.pop(&addr));
            let mut descriptor = if cfg!(target_os = "windows") {
                let mut shell = Command::new("cmd");
                shell.args(["/C", command]);
                shell
            } else {
                let mut shell = Command::new("sh");
                shell.arg("-c").arg(command);
                shell
            };

            match descriptor.spawn() {
                Ok(child) => {
                    vm.op_push(
                        OpcodeValue::Raw(Value::Any(memory::alloc_value(child))),
                        table,
                    )?;
                }
                Err(_e) => {
                    vm.push(Value::Null);
                }
            };

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "system@process_wait",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let child = utils::expect_any(&addr, vm.pop(&addr), None);
            let child: Option<&mut std::process::Child> = (*child).downcast_mut();

            match child {
                Some(ch) => {
                    let value = ch.wait();
                    if should_push {
                        match value {
                            Ok(status) => {
                                vm.push(Value::Int(status.code().unwrap_or(0) as _));
                            }
                            Err(e) => {
                                vm.push(Value::Null);
                            }
                        }
                    }
                }
                None => {
                    error!(Error::new(
                        addr.clone(),
                        "the inner raw value is not a `std::process::Child`",
                        "please file an issue at https://github.com/vyacheslavhere/watt"
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
        "system@process_terminate",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let child = utils::expect_any(&addr, vm.pop(&addr), None);
            let child: Option<&mut std::process::Child> = (*child).downcast_mut();

            match child {
                Some(ch) => {
                    let _ = ch.kill();
                }
                None => {
                    error!(Error::new(
                        addr.clone(),
                        "The inner raw value is not a `std::process::Child`",
                        "please file an issue at https://github.com/vyacheslavhere/watt"
                    ))
                }
            }

            if should_push {
                vm.push(Value::Null);
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "system@process_id",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let child = utils::expect_any(&addr, vm.pop(&addr), None);
            let child: Option<&mut std::process::Child> = (*child).downcast_mut();

            match child {
                Some(ch) => {
                    if should_push {
                        vm.push(Value::Int(ch.id() as _));
                    }
                }
                None => {
                    error!(Error::new(
                        addr.clone(),
                        "The inner raw value is not a `std::process::Child`",
                        "please file an issue at https://github.com/vyacheslavhere/watt"
                    ))
                }
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        0,
        "system@get_osname",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            if should_push {
                vm.op_push(OpcodeValue::String(std::env::consts::OS.to_string()), table)?;
            }

            Ok(())
        },
    );
    // успех
    Ok(())
}
