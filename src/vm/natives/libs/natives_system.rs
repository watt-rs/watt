use sysinfo::System;

// импорты
use crate::error;
use crate::errors::errors::Error;
use crate::lexer::address::Address;
use crate::vm::memory::memory;
use crate::vm::natives::libs::{sysinfo_provider, utils};
use crate::vm::natives::natives;
use crate::vm::table::Table;
use crate::vm::values::Value;
use crate::vm::vm::VM;

// провайд
#[allow(unused_variables)]
pub unsafe fn provide(built_in_address: Address, vm: &mut VM) -> Result<(), Error> {
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "system@getenv",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            if !should_push {
                vm.push(Value::Null);
                return Ok(());
            }

            let env_key = &*utils::expect_string(addr.clone(), vm.pop(&addr)?, None);

            match std::env::vars().find(|x| &x.0 == env_key) {
                Some((key, value)) => {
                    vm.push(Value::String(memory::alloc_value(value.clone())));
                }
                None => {
                    vm.push(Value::Null);
                }
            };

            Ok(())
        },
    );

    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "system@setenv",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let env_value = &*utils::expect_string(addr.clone(), vm.pop(&addr)?, None);
            let env_key = &*utils::expect_string(addr.clone(), vm.pop(&addr)?, None);

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
                vm.push(Value::Null);
                return Ok(());
            }

            match std::env::current_dir() {
                Ok(cwd) => {
                    let path = cwd.to_str().map(|x| x.to_string());

                    match path {
                        Some(p) => {
                            vm.push(Value::String(memory::alloc_value(p)));
                        }
                        None => {
                            vm.push(Value::Null);
                        }
                    }
                }
                Err(e) => {
                    vm.push(Value::Null);
                }
            }

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
                vm.push(Value::Null);
                return Ok(());
            }

            let args: Vec<Value> = std::env::args()
                .skip(1)
                .map(|x| Value::String(memory::alloc_value(x)))
                .collect();

            let watt_list = Value::List(memory::alloc_value(args));

            vm.push(watt_list);

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
                vm.push(Value::Null);
                return Ok(());
            }

            let sysinfo = System::new_with_specifics(
                sysinfo::RefreshKind::nothing()
                    .with_memory(sysinfo::MemoryRefreshKind::everything()),
            );

            vm.push(Value::Int(sysinfo.total_memory() as _));

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
                vm.push(Value::Null);
                return Ok(());
            }

            let sysinfo = System::new_with_specifics(
                sysinfo::RefreshKind::nothing()
                    .with_memory(sysinfo::MemoryRefreshKind::everything()),
            );

            vm.push(Value::Int(sysinfo.used_memory() as _));

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
                vm.push(Value::Null);
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
        "system@process_id",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            vm.push(Value::Int(std::process::id() as _));

            Ok(())
        },
    );

    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "system@process_exit",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let code = utils::expect_int(addr.clone(), vm.pop(&addr)?, None);

            std::process::exit(code as _);
        },
    );

    // успех
    Ok(())
}
