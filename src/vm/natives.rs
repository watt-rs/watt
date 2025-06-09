use crate::errors::Error;
use crate::lexer::address::Address;
use crate::vm::memory;
use crate::vm::table::Table;
use crate::vm::values::{Native, Symbol, Value};
use crate::vm::vm::{VM};

pub unsafe fn provide(vm: &mut VM) -> Result<(), Error> {
    // нативный адрес
    let mut natives_address = Address::new(0, "builtins".to_string());
    // нативные функции
    (*vm.globals).define(
        natives_address.clone(),
        "println".to_string(),
        Value::Native(
            memory::alloc_value(
                Native::new(
                    Symbol::new(
                        "println".to_string(),
                        "builtin:println".to_string()
                    ),
                    1,
                    |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
                        println!("{:?}", vm.pop(addr.clone())?);
                        if should_push {
                            vm.push(Value::Null)
                        }
                        Ok(())
                    }
                )
            )
        )
    )?;
    (*vm.globals).define(
        natives_address.clone(),
        "gc".to_string(),
        Value::Native(
            memory::alloc_value(
                Native::new(
                    Symbol::new(
                        "gc".to_string(),
                        "builtin:gc".to_string()
                    ),
                    0,
                    |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
                        vm.gc_invoke(table);
                        if should_push {
                            vm.push(Value::Null)
                        }
                        Ok(())
                    }
                )
            )
        )
    )?;
    // успех
    Ok(())
}