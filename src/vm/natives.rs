// импорты
use crate::errors::errors::Error;
use crate::lexer::address::Address;
use crate::vm::bytecode::Chunk;
use crate::vm::memory::memory;
use crate::vm::table::Table;
use crate::vm::values::{Native, Symbol, Value};
use crate::vm::vm::{VM};

// провайд билтинов
pub unsafe fn provide_builtins(vm: &mut VM) -> Result<(), Error> {
    // нативный адрес
    let natives_address = Address::new(
        0,
        0,
        "builtins".to_string(),
        "fun ... (..., ..., n) {".to_string()
    );
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
    // todo delete this temp native:
    (*vm.globals).define(
        natives_address.clone(),
        "thread".to_string(),
        Value::Native(
            memory::alloc_value(
                Native::new(
                    Symbol::new(
                        "thread".to_string(),
                        "builtin:thread".to_string()
                    ),
                    1,
                    |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
                        // функция
                        let value = vm.pop(addr.clone())?;
                        // запуск
                        if let Value::Fn(function) = value {
                            unsafe {
                                vm.start_thread(addr, function, table, Box::new(Chunk::new(vec![])));
                            }
                        }
                        // нулл
                        if should_push {
                            vm.push(Value::Null);
                        }
                        // успех
                        Ok(())
                    }
                )
            )
        )
    )?;
    // успех
    Ok(())
}