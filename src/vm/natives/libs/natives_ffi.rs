// импорты
use crate::error;
use crate::errors::errors::Error;
use crate::lexer::address::Address;
use crate::vm::bytecode::OpcodeValue;
use crate::vm::memory::memory::{self, alloc_value};
use crate::vm::natives::natives;
use crate::vm::table::Table;
use crate::vm::values::{Value};
use crate::vm::vm::VM;

use libloading;

// провайд
#[allow(unused_variables)]
pub unsafe fn provide(built_in_address: Address, vm: &mut VM) -> Result<(), Error> {
    // функции
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "ffi@load".to_string(),
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let path = match vm.pop(&addr) {
                Ok(Value::String(a)) => {
                    &*a
                },
                Ok(a) => {
                    error!(
                        Error::new(
                            addr.clone(),
                            format!("The `{:?}` value is not a String!", a),
                            "pass a path in string format"
                        )
                    );
                }
                Err(e) => {
                    todo!("Error: {e:?}")
                }
            };

            if should_push {
                let library = match libloading::Library::new(&path) {
                    Ok(lib) => {
                        lib
                    },
                    Err(e) => {
                        vm.op_push(
                            OpcodeValue::String(e.to_string()),
                            table
                        )?;

                        return Ok(());
                    }
                };
                
                let mem = alloc_value(library);

                vm.op_push(
                    OpcodeValue::Raw(Value::Any(mem)),
                    table
                )?;
            }

            Ok(())
        }
    );

    natives::provide(
        vm,
        built_in_address.clone(),
        3,
        "ffi@call".to_string(),
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let args = match vm.pop(&addr) {
                Ok(Value::Instance(a)) => {
                    &*a
                },
                Ok(a) => {
                    error!(
                        Error::new(
                            addr.clone(),
                            format!("Arguments should be in a List! (got {:?})", a),
                            "check your code"
                        )
                    );
                }
                Err(e) => {
                    todo!("Error: {e:?}")
                }
            };

            let fn_name = match vm.pop(&addr) {
                Ok(Value::String(a)) => {
                    &*a
                },
                Ok(a) => {
                    error!(
                        Error::new(
                            addr.clone(),
                            format!("The `{:?}` value is not a String!", a),
                            "pass a path in string format"
                        )
                    );
                }
                Err(e) => {
                    todo!("Error: {e:?}")
                }
            };

            let lib = match vm.pop(&addr) {
                Ok(Value::Any(a)) => {
                    let instance = &*a;

                    let n: &libloading::Library = instance.downcast_ref().unwrap();

                    n
                },
                Ok(a) => {
                    error!(
                        Error::new(
                            addr.clone(),
                            format!("The `{:?}` value is not a pointer to libloading::Library!", a),
                            "are you trying to access natives manually?"
                        )
                    );
                }
                Err(e) => {
                    todo!("Error: {e:?}")
                }
            };

            let function: libloading::Symbol<unsafe extern "C" fn()> = match lib.get(fn_name.as_bytes()) {
                Ok(sym) => sym,
                Err(e) => {
                    vm.op_push(
                        OpcodeValue::Raw(Value::Null),
                        table
                    )?;

                    return Ok(());
                }
            };

            function();

            if should_push {
                vm.op_push(
                    OpcodeValue::Raw(Value::Null),
                    table
                )?;
                // vm.op_push(
                //     OpcodeValue::Raw(Value::Any(mem)),
                //     table
                // )?;
            }

            Ok(())
        }
    );

    // успех
    Ok(())
}