// импорты
use std::io::{self, Write};
use crate::error;
use crate::errors::errors::Error;
use crate::lexer::address::Address;
use crate::vm::bytecode::OpcodeValue;
use crate::vm::natives::natives;
use crate::vm::table::Table;
use crate::vm::values::{Value};
use crate::vm::vm::VM;

// провайд
#[allow(unused_variables)]
pub unsafe fn provide(built_in_address: Address, vm: &mut VM) -> Result<(), Error> {
    // функции
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "io@println",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            println!("{:?}", vm.pop(&addr)?);

            if should_push {
                vm.push(Value::Null)
            }
            
            Ok(())
        }
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "io@print",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            print!("{:?}", vm.pop(&addr)?);

            if should_push {
                vm.push(Value::Null)
            }

            Ok(())
        }
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        0,
        "io@flush",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            io::stdout().lock().flush().unwrap();

            if should_push {
                vm.push(Value::Null)
            }
            
            Ok(())
        }
    );
    natives::provide(
        vm,
        built_in_address,
        0,
        "io@input",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            // инпут
            let mut input: String = String::new();
            if let Err(e) = io::stdin()
                .read_line(&mut input) {
                error!(Error::own_text(
                    addr,
                    format!("io error in input: {}", e),
                    "check your code"
                ))
            }
            // если нужен пуш
            if should_push {
                vm.op_push(
                    OpcodeValue::String(
                        input
                    ),
                    table
                )?;
            }
            // успех
            Ok(())
        }
    );
    // успех
    Ok(())
}