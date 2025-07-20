use crate::error;
// imports
use crate::errors::errors::Error;
use crate::lexer::address::Address;
use crate::vm::bytecode::OpcodeValue;
use crate::vm::memory::memory;
use crate::vm::natives::libs::utils;
use crate::vm::natives::natives;
use crate::vm::table::Table;
use crate::vm::values::Value;
use crate::vm::vm::VM;

/// Provides
#[allow(unused_variables)]
#[allow(clippy::manual_range_contains)]
pub unsafe fn provide(built_in_address: &Address, vm: &mut VM) -> Result<(), Error> {
    natives::provide(
        vm,
        built_in_address.clone(),
        3,
        "strings@replace",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let to = utils::expect_cloned_string(addr.clone(), vm.pop(&addr)?, None);
            let what = utils::expect_cloned_string(addr.clone(), vm.pop(&addr)?, None);
            let string = utils::expect_string(addr.clone(), vm.pop(&addr)?, None);
            if should_push {
                let result = (*string).replace(what.as_str(), to.as_str());
                vm.op_push(OpcodeValue::String(result), table)?;
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        4,
        "strings@replace_n",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let n = utils::expect_int(addr.clone(), vm.pop(&addr)?, None);
            let to = utils::expect_cloned_string(addr.clone(), vm.pop(&addr)?, None);
            let what = utils::expect_cloned_string(addr.clone(), vm.pop(&addr)?, None);
            let string = utils::expect_string(addr.clone(), vm.pop(&addr)?, None);
            if should_push {
                let result = (*string).replacen(what.as_str(), to.as_str(), n as usize);
                vm.op_push(OpcodeValue::String(result), table)?;
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        4,
        "strings@replace_range",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let a = utils::expect_int(addr.clone(), vm.pop(&addr)?, None);
            let b = utils::expect_int(addr.clone(), vm.pop(&addr)?, None);
            let to = utils::expect_cloned_string(addr.clone(), vm.pop(&addr)?, None);
            let mut string = (*utils::expect_string(addr.clone(), vm.pop(&addr)?, None)).clone();
            if should_push {
                string.replace_range((a as usize)..(b as usize), to.as_str());
                vm.op_push(OpcodeValue::String(string), table)?;
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "strings@char_at",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let i = utils::expect_int(addr.clone(), vm.pop(&addr)?, None);
            let string = utils::expect_string(addr.clone(), vm.pop(&addr)?, None);
            if should_push {
                let result = (*string).chars().nth(i as usize).unwrap();
                vm.op_push(OpcodeValue::String(result.to_string()), table)?;
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "strings@chars",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let string = utils::expect_string(addr.clone(), vm.pop(&addr)?, None);
            if should_push {
                let result: Vec<Value> = (*string)
                    .chars()
                    .map(|ch| {
                        let string = Value::String(memory::alloc_value(ch.to_string()));
                        vm.gc_guard(string);
                        vm.gc_register(string, table);
                        string
                    })
                    .collect();

                // unguarding strings
                for _ in 0..(*string).len() {
                    vm.gc_unguard();
                }
                // safety of strings will not be erased
                // guaranteed by list marking if gc will invoke.
                let raw_list = Value::List(memory::alloc_value(result));
                vm.op_push(OpcodeValue::Raw(raw_list), table)?;
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "strings@trim",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let string = utils::expect_string(addr.clone(), vm.pop(&addr)?, None);
            if should_push {
                vm.op_push(OpcodeValue::String((*string).trim().to_string()), table)?;
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "strings@split",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let delimiter = utils::expect_cloned_string(addr.clone(), vm.pop(&addr)?, None);
            let string = utils::expect_string(addr.clone(), vm.pop(&addr)?, None);
            if should_push {
                let result: Vec<Value> = (*string)
                    .split(delimiter.as_str())
                    .map(|str| {
                        let string = Value::String(memory::alloc_value(str.to_string()));
                        vm.gc_guard(string);
                        vm.gc_register(string, table);
                        string
                    })
                    .collect();

                // unguarding strings
                for _ in 0..(*string).len() {
                    vm.gc_unguard();
                }
                // safety of strings will not be erased
                // guaranteed by list marking if gc will invoke.
                let raw_list = Value::List(memory::alloc_value(result));
                vm.op_push(OpcodeValue::Raw(raw_list), table)?;
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        3,
        "strings@substring",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let to = utils::expect_int(addr.clone(), vm.pop(&addr)?, None);
            let from = utils::expect_int(addr.clone(), vm.pop(&addr)?, None);
            let string = utils::expect_string(addr.clone(), vm.pop(&addr)?, None);
            if should_push {
                let result: String = (*string)[(from as usize)..(to as usize)].to_string();
                vm.op_push(OpcodeValue::String(result), table)?;
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "strings@contains",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let value = utils::expect_cloned_string(addr.clone(), vm.pop(&addr)?, None);
            let string = utils::expect_string(addr.clone(), vm.pop(&addr)?, None);
            if should_push {
                vm.op_push(OpcodeValue::Bool((*string).contains(value.as_str())), table)?;
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "strings@find",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let raw_ch = utils::expect_cloned_string(addr.clone(), vm.pop(&addr)?, None);
            let string = utils::expect_string(addr.clone(), vm.pop(&addr)?, None);
            if raw_ch.len() != 1 {
                error!(Error::own_hint(
                    addr.clone(),
                    "could not represent string as char.",
                    format!("string: {raw_ch}")
                ))
            }
            let ch = raw_ch.chars().next().unwrap();
            if should_push {
                match (*string).chars().position(|char| char == ch) {
                    None => {
                        vm.op_push(OpcodeValue::Int(-1), table)?;
                    }
                    Some(i) => {
                        vm.op_push(OpcodeValue::Int(i as i64), table)?;
                    }
                }
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "strings@rfind",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let raw_ch = utils::expect_cloned_string(addr.clone(), vm.pop(&addr)?, None);
            let string = utils::expect_string(addr.clone(), vm.pop(&addr)?, None);
            if raw_ch.len() != 1 {
                error!(Error::own_hint(
                    addr.clone(),
                    "could not represent string as char.",
                    format!("string: {raw_ch:?}")
                ))
            }
            let ch = raw_ch.chars().next().unwrap();
            if should_push {
                match (*string).chars().rev().position(|char| char == ch) {
                    None => {
                        vm.op_push(OpcodeValue::Int(-1), table)?;
                    }
                    Some(i) => {
                        vm.op_push(OpcodeValue::Int(i as i64), table)?;
                    }
                }
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "strings@push",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let what = utils::expect_cloned_string(addr.clone(), vm.pop(&addr)?, None);
            let target = utils::expect_string(addr.clone(), vm.pop(&addr)?, None) as *mut String;
            (*target).push_str(what.as_str());
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
        "strings@length",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let string = utils::expect_cloned_string(addr.clone(), vm.pop(&addr)?, None);
            if should_push {
                vm.push(Value::Int(string.len() as i64));
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "char@is_ascii_letter",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let raw_ch = utils::expect_cloned_string(addr.clone(), vm.pop(&addr)?, None);

            if should_push {
                vm.push(Value::Bool(raw_ch.is_ascii()));
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "char@is_digit",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let radix = utils::expect_int(addr.clone(), vm.pop(&addr)?, None);
            let raw_ch = utils::expect_cloned_string(addr.clone(), vm.pop(&addr)?, None);

            // radix rust bounds
            if radix > 36 || radix < 2 {
                error!(Error::own_text(
                    addr.clone(),
                    format!("invalid radix: {radix}"),
                    "radix should be in 2..36 range."
                ))
            }
            if raw_ch.len() != 1 {
                error!(Error::own_hint(
                    addr.clone(),
                    "could not represent string as char.",
                    format!("string: {raw_ch:?}")
                ))
            }

            if should_push {
                let ch = raw_ch.chars().next().unwrap();
                vm.push(Value::Bool(ch.is_digit(radix as u32)));
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "char@as_int",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let raw_ch = utils::expect_cloned_string(addr.clone(), vm.pop(&addr)?, None);

            if raw_ch.len() != 1 {
                error!(Error::own_hint(
                    addr.clone(),
                    "could not represent string as char.",
                    format!("string: {raw_ch:?}")
                ))
            }

            if should_push {
                let ch = raw_ch.chars().next().unwrap();
                vm.push(Value::Int(ch as i64));
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "strings@lower",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let string = utils::expect_string(addr.clone(), vm.pop(&addr)?, None);
            if should_push {
                vm.op_push(OpcodeValue::String((*string).to_lowercase()), table)?;
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "strings@upper",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let string = utils::expect_string(addr.clone(), vm.pop(&addr)?, None);
            if should_push {
                vm.op_push(OpcodeValue::String((*string).to_uppercase()), table)?;
            }
            Ok(())
        },
    );
    Ok(())
}
