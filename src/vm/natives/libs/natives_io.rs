use std::io;
use crate::error;
use crate::errors::errors::Error;
use crate::lexer::address::Address;
use crate::vm::memory::memory;
use crate::vm::natives::natives;
use crate::vm::table::Table;
use crate::vm::values::{FnOwner, Value};
use crate::vm::vm::VM;

// провайд
pub unsafe fn provide(built_in_address: Address, vm: &mut VM) -> Result<(), Error> {
    // функции
    natives::provide(
        vm,
        built_in_address.clone(),
        "io@println".to_string(),
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table, owner: *mut FnOwner| {
            println!("{:?}", vm.pop(addr.clone())?);
            if should_push {
                vm.push(Value::Null)
            }
            Ok(())
        }
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        "io@print".to_string(),
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table, owner: *mut FnOwner| {
            print!("{:?}", vm.pop(addr.clone())?);
            if should_push {
                vm.push(Value::Null)
            }
            Ok(())
        }
    );
    natives::provide(
        vm,
        built_in_address,
        "io@input".to_string(),
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table, owner: *mut FnOwner| {
            // инпут
            let mut input: String = String::new();
            if let Err(e) = io::stdin()
                .read_line(&mut input) {
                error!(Error::new(
                    addr,
                    format!("io error in input: {}", e),
                    "check your code".to_string()
                ))
            }
            // если нужен пуш
            if should_push {
                vm.push(Value::String(
                    memory::alloc_value(input)
                ))
            }
            // успех
            Ok(())
        }
    );
    // успех
    Ok(())
}