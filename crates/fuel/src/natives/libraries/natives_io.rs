// imports
use crate::bytecode::OpcodeValue;
use crate::natives::natives;
use crate::values::Value;
use crate::vm::VirtualMachine;
use oil_common::address::Address;
use oil_common::{error, errors::Error};
use std::io::{self, Write};

/// Provides
#[allow(unused_variables)]
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe fn provide(built_in_address: &Address, vm: &mut VirtualMachine) -> Result<(), Error> {
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "io@println",
        |vm: &mut VirtualMachine, addr: Address, should_push: bool| {
            println!("{:?}", vm.pop(&addr));

            if should_push {
                vm.push(Value::Null)
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "io@print",
        |vm: &mut VirtualMachine, addr: Address, should_push: bool| {
            print!("{:?}", vm.pop(&addr));

            if should_push {
                vm.push(Value::Null)
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        0,
        "io@flush",
        |vm: &mut VirtualMachine, addr: Address, should_push: bool| {
            io::stdout().lock().flush().unwrap();

            if should_push {
                vm.push(Value::Null)
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        0,
        "io@input",
        |vm: &mut VirtualMachine, addr: Address, should_push: bool| {
            let mut input: String = String::new();

            if let Err(e) = io::stdin().read_line(&mut input) {
                error!(Error::own_text(
                    addr,
                    format!("io error in input: {e}"),
                    "check your code"
                ))
            }

            if should_push {
                vm.op_push(OpcodeValue::String(input))?;
            }

            Ok(())
        },
    );
    Ok(())
}
