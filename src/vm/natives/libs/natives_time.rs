// импорты
use chrono::{DateTime, Datelike, Local, Timelike};
use crate::error;
use crate::errors::errors::Error;
use crate::lexer::address::Address;
use crate::vm::bytecode::OpcodeValue;
use crate::vm::memory::memory;
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
        0,
        "time@now".to_string(),
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            if should_push {
                vm.op_push(OpcodeValue::Raw(Value::Any(
                    memory::alloc_value(Local::now())
                )), table)?;
            }
            Ok(())
        }
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "timestamp@mills".to_string(),
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            match vm.pop(&addr) {
                Ok(ok) => {
                    match ok {
                        Value::Any(val) => {
                            match (*val).downcast_mut::<DateTime<Local>>() {
                                Some(time) => {
                                    if should_push {
                                        vm.push(Value::Int(time.timestamp_millis()));
                                    }
                                },
                                None => {
                                    error!(Error::new(
                                        addr,
                                        format!("invalid timestamp: {:?}", ok),
                                        "check your code.",
                                    ))
                                }
                            }
                        }
                        _ => {
                            error!(Error::new(
                                addr,
                                format!("invalid timestamp: {:?}", ok),
                                "check your code.",
                            ))
                        }
                    }
                }
                Err(flow) => {
                    return Err(flow)
                }
            }
            if should_push {

            }
            Ok(())
        }
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "timestamp@second".to_string(),
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            match vm.pop(&addr) {
                Ok(ok) => {
                    match ok {
                        Value::Any(val) => {
                            match (*val).downcast_mut::<DateTime<Local>>() {
                                Some(time) => {
                                    if should_push {
                                        vm.push(Value::Int(time.second().into()));
                                    }
                                },
                                None => {
                                    error!(Error::new(
                                        addr,
                                        format!("invalid timestamp: {:?}", ok),
                                        "check your code.",
                                    ))
                                }
                            }
                        }
                        _ => {
                            error!(Error::new(
                                addr,
                                format!("invalid timestamp: {:?}", ok),
                                "check your code.",
                            ))
                        }
                    }
                }
                Err(flow) => {
                    return Err(flow)
                }
            }
            if should_push {

            }
            Ok(())
        }
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "timestamp@minute".to_string(),
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            match vm.pop(&addr) {
                Ok(ok) => {
                    match ok {
                        Value::Any(val) => {
                            match (*val).downcast_mut::<DateTime<Local>>() {
                                Some(time) => {
                                    if should_push {
                                        vm.push(Value::Int(time.minute().into()));
                                    }
                                },
                                None => {
                                    error!(Error::new(
                                        addr,
                                        format!("invalid timestamp: {:?}", ok),
                                        "check your code.",
                                    ))
                                }
                            }
                        }
                        _ => {
                            error!(Error::new(
                                addr,
                                format!("invalid timestamp: {:?}", ok),
                                "check your code.",
                            ))
                        }
                    }
                }
                Err(flow) => {
                    return Err(flow)
                }
            }
            if should_push {

            }
            Ok(())
        }
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "timestamp@hour".to_string(),
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            match vm.pop(&addr) {
                Ok(ok) => {
                    match ok {
                        Value::Any(val) => {
                            match (*val).downcast_mut::<DateTime<Local>>() {
                                Some(time) => {
                                    if should_push {
                                        vm.push(Value::Int(time.hour().into()));
                                    }
                                },
                                None => {
                                    error!(Error::new(
                                        addr,
                                        format!("invalid timestamp: {ok:?}"),
                                        "check your code.",
                                    ))
                                }
                            }
                        }
                        _ => {
                            error!(Error::new(
                                addr,
                                format!("invalid timestamp: {ok:?}"),
                                "check your code.",
                            ))
                        }
                    }
                }
                Err(flow) => {
                    return Err(flow)
                }
            }
            if should_push {

            }
            Ok(())
        }
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "timestamp@day".to_string(),
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            match vm.pop(&addr) {
                Ok(ok) => {
                    match ok {
                        Value::Any(val) => {
                            match (*val).downcast_mut::<DateTime<Local>>() {
                                Some(time) => {
                                    if should_push {
                                        vm.push(Value::Int(time.day().into()));
                                    }
                                },
                                None => {
                                    error!(Error::new(
                                        addr,
                                        format!("invalid timestamp: {:?}", ok),
                                        "check your code.",
                                    ))
                                }
                            }
                        }
                        _ => {
                            error!(Error::new(
                                addr,
                                format!("invalid timestamp: {:?}", ok),
                                "check your code.",
                            ))
                        }
                    }
                }
                Err(flow) => {
                    return Err(flow)
                }
            }
            if should_push {

            }
            Ok(())
        }
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "timestamp@year".to_string(),
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            match vm.pop(&addr) {
                Ok(ok) => {
                    match ok {
                        Value::Any(val) => {
                            match (*val).downcast_mut::<DateTime<Local>>() {
                                Some(time) => {
                                    if should_push {
                                        vm.push(Value::Int(time.year().into()));
                                    }
                                },
                                None => {
                                    error!(Error::new(
                                        addr,
                                        format!("invalid timestamp: {:?}", ok),
                                        "check your code.",
                                    ))
                                }
                            }
                        }
                        _ => {
                            error!(Error::new(
                                addr,
                                format!("invalid timestamp: {:?}", ok),
                                "check your code.",
                            ))
                        }
                    }
                }
                Err(flow) => {
                    return Err(flow)
                }
            }
            if should_push {

            }
            Ok(())
        }
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "timestamp@month".to_string(),
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            match vm.pop(&addr) {
                Ok(ok) => {
                    match ok {
                        Value::Any(val) => {
                            match (*val).downcast_mut::<DateTime<Local>>() {
                                Some(time) => {
                                    if should_push {
                                        vm.push(Value::Int(time.month().into()));
                                    }
                                },
                                None => {
                                    error!(Error::new(
                                        addr,
                                        format!("invalid timestamp: {:?}", ok),
                                        "check your code.",
                                    ))
                                }
                            }
                        }
                        _ => {
                            error!(Error::new(
                                addr,
                                format!("invalid timestamp: {:?}", ok),
                                "check your code.",
                            ))
                        }
                    }
                }
                Err(flow) => {
                    return Err(flow)
                }
            }
            if should_push {

            }
            Ok(())
        }
    );
    // успех
    Ok(())
}