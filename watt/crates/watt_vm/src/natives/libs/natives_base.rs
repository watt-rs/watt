// imports
use crate::bytecode::OpcodeValue;
use crate::memory::gc::Gc;
use crate::natives::natives;
use crate::natives::utils;
use crate::table::Table;
use crate::values::Value;
use crate::vm::VM;
use watt_common::address::Address;
use watt_common::{error, errors::Error};

/// Provides
#[allow(unused_variables)]
pub unsafe fn provide(built_in_address: &Address, vm: &mut VM) -> Result<(), Error> {
    // panic
    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "base@panic",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            // hint and error texts
            let hint = utils::expect_cloned_string(&addr, vm.pop(&addr));
            let error = utils::expect_cloned_string(&addr, vm.pop(&addr));
            // raising an error
            error!(Error::own(addr.clone(), error, hint));
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "base@typeof",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let value = vm.pop(&addr);

            if !should_push {
                return Ok(());
            }

            match value {
                Value::Float(_) => {
                    vm.op_push(OpcodeValue::String("f64".to_string()))?;
                }
                Value::Int(_) => {
                    vm.op_push(OpcodeValue::String("i64".to_string()))?;
                }
                Value::String(_) => {
                    vm.op_push(OpcodeValue::String("string".to_string()))?;
                }
                Value::Bool(_) => {
                    vm.op_push(OpcodeValue::String("bool".to_string()))?;
                }
                Value::Type(_) => {
                    vm.op_push(OpcodeValue::String("type".to_string()))?;
                }
                Value::Fn(_) => {
                    vm.op_push(OpcodeValue::String("fn".to_string()))?;
                }
                Value::Native(_) => {
                    vm.op_push(OpcodeValue::String("native".to_string()))?;
                }
                Value::Instance(i) => {
                    vm.op_push(OpcodeValue::String((*(*i).t).name.clone()))?;
                }
                Value::Unit(u) => {
                    vm.op_push(OpcodeValue::String((*u).name.clone()))?;
                }
                Value::Trait(t) => {
                    vm.op_push(OpcodeValue::String((*t).name.clone()))?;
                }
                Value::List(l) => {
                    vm.op_push(OpcodeValue::String("list".to_string()))?;
                }
                Value::Null => {
                    vm.op_push(OpcodeValue::String("null".to_string()))?;
                }
                Value::Any(_) => {
                    vm.op_push(OpcodeValue::String("any".to_string()))?;
                }
                Value::Module(m) => {}
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "base@full_typeof",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let value = vm.pop(&addr);

            if !should_push {
                return Ok(());
            }

            match value {
                Value::Float(_) => {
                    vm.op_push(OpcodeValue::String("watt:f64".to_string()))?;
                }
                Value::Int(_) => {
                    vm.op_push(OpcodeValue::String("watt:i64".to_string()))?;
                }
                Value::String(_) => {
                    vm.op_push(OpcodeValue::String("watt:string".to_string()))?;
                }
                Value::Bool(_) => {
                    vm.op_push(OpcodeValue::String("watt:bool".to_string()))?;
                }
                Value::Type(_) => {
                    vm.op_push(OpcodeValue::String("watt:type".to_string()))?;
                }
                Value::Fn(_) => {
                    vm.op_push(OpcodeValue::String("watt:fn".to_string()))?;
                }
                Value::Native(_) => {
                    vm.op_push(OpcodeValue::String("watt:native".to_string()))?;
                }
                Value::Instance(i) => {
                    let name = (*(*i).t).name.clone();
                    vm.op_push(OpcodeValue::String(name))?;
                }
                Value::Unit(u) => {
                    let name = (*u).name.clone();
                    vm.op_push(OpcodeValue::String(name))?;
                }
                Value::Trait(t) => {
                    let name = (*t).name.clone();
                    vm.op_push(OpcodeValue::String(name))?;
                }
                Value::List(l) => {
                    vm.op_push(OpcodeValue::String("watt:list".to_string()))?;
                }
                Value::Null => {
                    vm.op_push(OpcodeValue::String("watt:null".to_string()))?;
                }
                Value::Any(any) => {
                    vm.op_push(OpcodeValue::String("watt:any".to_string()))?;
                }
                Value::Module(_) => {
                    vm.op_push(OpcodeValue::String("watt:module".to_string()))?;
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
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let value = vm.pop(&addr);

            if !should_push {
                return Ok(());
            }

            match value {
                Value::Instance(_) => {
                    vm.op_push(OpcodeValue::Bool(true))?;
                }
                _ => {
                    vm.op_push(OpcodeValue::Bool(false))?;
                }
            }
            Ok(())
        },
    );
    Ok(())
}
