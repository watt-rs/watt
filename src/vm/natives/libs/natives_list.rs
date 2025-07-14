// imports
use crate::errors::errors::Error;
use crate::lexer::address::Address;
use crate::vm::natives::natives;
use crate::vm::table::Table;
use crate::vm::values::{Value};
use crate::vm::vm::VM;
use crate::error;
use crate::vm::bytecode::OpcodeValue;
use crate::vm::memory::memory;

/// Provides
#[allow(unused_variables)]
#[allow(dangerous_implicit_autorefs)]
pub unsafe fn provide(built_in_address: &Address, vm: &mut VM) -> Result<(), Error> {
    natives::provide(
        vm,
        built_in_address.clone(),
        0,
        "list@make",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            if should_push {
                let list = memory::alloc_value(Vec::<Value>::new());
                vm.op_push(OpcodeValue::Raw(
                    Value::List(
                        list
                    )
                ), table)?;
            }
            Ok(())
        }
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "list@add",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let value = vm.pop(&addr).unwrap();
            let list_value = vm.pop(&addr).unwrap();
            
            if let Value::List(list) = list_value {
                (*list).push(value);
            }
            else {
                error!(Error::own_text(
                    addr.clone(),
                    format!("could not add element to {:?}, not a list", list_value),
                    "check your code"
                ));
            }
            if should_push {
                vm.push(Value::Null)
            }
            Ok(())
        }
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        3,
        "list@set",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let value = vm.pop(&addr).unwrap();
            let index_value = vm.pop(&addr).unwrap();
            let list_value = vm.pop(&addr).unwrap();

            if let Value::List(list) = list_value {
                if let Value::Int(index) = index_value {
                    (*list)[index as usize] = value;
                }
                else {
                    error!(Error::own_text(
                        addr.clone(),
                        format!(
                            "could not set element to {:?}, index is {:?}, not an i64",
                            list_value, index_value
                        ),
                        "check your code"
                    ))
                }
            }
            else {
                error!(Error::own_text(
                    addr.clone(),
                    format!("could not set element in {:?}, not a list", list_value),
                    "check your code"
                ));
            }
            if should_push {
                vm.push(Value::Null)
            }
            // успех
            Ok(())
        }
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "list@get",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let index_value = vm.pop(&addr).unwrap();
            let list_value = vm.pop(&addr).unwrap();

            if let Value::List(list) = list_value {
                if let Value::Int(index) = index_value {
                    // bounds checking
                    if index < 0 || index as usize >= (*list).len() {
                        error!(Error::own_text(
                            addr.clone(),
                            format!("index {} out of bounds [0, {}]", index, (*list).len()),
                            "check your code"
                        ))
                    }
                    if should_push {
                        let value = *((*list).get(index as usize).unwrap());
                        vm.push(value);
                    }
                }
                else {
                    error!(Error::own_text(
                        addr.clone(),
                        format!(
                            "could not set element to {:?}, index is {:?}, not an i64",
                            list_value, index_value
                        ),
                        "check your code"
                    ))
                }
            }
            else {
                error!(Error::own_text(
                    addr.clone(),
                    format!("could not get element from {:?}, not a list", list_value),
                    "check your code"
                ));
            }

            Ok(())
        }
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "list@delete_at",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let index_value = vm.pop(&addr).unwrap();
            let list_value = vm.pop(&addr).unwrap();

            if let Value::List(list) = list_value {
                if let Value::Int(index) = index_value {
                    // bounds checking
                    if index < 0 || index as usize > (*list).len() {
                        error!(Error::own_text(
                            addr.clone(),
                            format!("index {} out of bounds [0, {}]", index, (*list).len()),
                            "check your code"
                        ))
                    }
                    
                    (*list).remove(index as usize);
                    
                    if should_push {
                        vm.push(Value::Null)
                    }
                }
                else {
                    error!(Error::own_text(
                        addr.clone(),
                        format!(
                            "could not set element to {:?}, index is {:?}, not a i64",
                            list_value, index_value
                        ),
                        "check your code"
                    ))
                }
            }
            else {
                error!(Error::own_text(
                    addr.clone(),
                    format!("could not get element from {:?}, not a list", list_value),
                    "check your code"
                ));
            }
            // успех
            Ok(())
        }
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "list@delete",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let value = vm.pop(&addr).unwrap();
            let list_value = vm.pop(&addr).unwrap();

            if let Value::List(list) = list_value {
                for (index, element) in (*list).iter().enumerate() {
                    if *element == value {
                        (*list).remove(index);
                        return Ok(());
                    }
                }
            }
            else {
                error!(Error::own_text(
                    addr.clone(),
                    format!("could not get element from {:?}, not a list", list_value),
                    "check your code"
                ));
            }

            Ok(())
        }
    );    
    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "list@index_of",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let value = vm.pop(&addr).unwrap();
            let list_value = vm.pop(&addr).unwrap();
            
            if let Value::List(list) = list_value {
                if should_push {
                    let position = (*list).iter().position(|v| *v == value);
                    vm.push(Value::Int(
                        position.unwrap_or(0) as i64
                    ))
                }
            }
            else {
                error!(Error::own_text(
                    addr.clone(),
                    format!("could not get element index from {:?}, not a list", list_value),
                    "check your code"
                ));
            }

            Ok(())
        }
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "list@length",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let list_value = vm.pop(&addr).unwrap();

            if let Value::List(list) = list_value {
                if should_push {
                    vm.push(
                        Value::Int(
                            (*list).len() as i64
                        )
                    );
                }
            }
            else {
                error!(Error::own_text(
                    addr.clone(),
                    format!("could not get len of {:?}, not a list", list_value),
                    "check your code"
                ));
            }

            Ok(())
        }
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "list@to_string",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let list_value = vm.pop(&addr).unwrap();

            if let Value::List(list) = list_value {
                if should_push {
                    vm.op_push(OpcodeValue::String(
                        format!("{:?}", *list)
                    ), table)?;
                }
            }
            else {
                error!(Error::own_text(
                    addr.clone(),
                    format!("could not use to_string for {:?}, not a list", list_value),
                    "check your code"
                ));
            }

            Ok(())
        }
    );
    Ok(())
}
