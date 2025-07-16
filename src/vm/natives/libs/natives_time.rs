// imports
use crate::error;
use crate::errors::errors::Error;
use crate::lexer::address::Address;
use crate::vm::bytecode::OpcodeValue;
use crate::vm::memory::memory;
use crate::vm::natives::libs::utils;
use crate::vm::natives::natives;
use crate::vm::table::Table;
use crate::vm::values::Value;
use crate::vm::vm::VM;
use chrono::{DateTime, Datelike, Local, Timelike};

/// Provides
#[allow(unused_variables)]
pub unsafe fn provide(built_in_address: &Address, vm: &mut VM) -> Result<(), Error> {
    natives::provide(
        vm,
        built_in_address.clone(),
        0,
        "time@now",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            if should_push {
                vm.op_push(
                    OpcodeValue::Raw(Value::Any(memory::alloc_value(Local::now()))),
                    table,
                )?;
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "timestamp@mills",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let timestamp_value = vm.pop(&addr.clone())?;
            let timestamp = utils::expect_any(
                addr.clone(),
                timestamp_value,
                Some(Error::own_text(
                    addr.clone(),
                    format!("not a timestamp: {timestamp_value:?}"),
                    "check your code.",
                )),
            );
            match (*timestamp).downcast_mut::<DateTime<Local>>() {
                Some(time) => {
                    if should_push {
                        vm.push(Value::Int(time.timestamp_millis()));
                    }
                }
                None => {
                    error!(Error::own_text(
                        addr,
                        format!("invalid timestamp: {timestamp:?}"),
                        "check your code.",
                    ))
                }
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "timestamp@second",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let timestamp_value = vm.pop(&addr.clone())?;
            let timestamp = utils::expect_any(
                addr.clone(),
                timestamp_value,
                Some(Error::own_text(
                    addr.clone(),
                    format!("not a timestamp: {timestamp_value:?}"),
                    "check your code.",
                )),
            );
            match (*timestamp).downcast_mut::<DateTime<Local>>() {
                Some(time) => {
                    if should_push {
                        vm.push(Value::Int(time.second().into()));
                    }
                }
                None => {
                    error!(Error::own_text(
                        addr,
                        format!("invalid timestamp: {timestamp:?}"),
                        "check your code.",
                    ))
                }
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "timestamp@minute",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let timestamp_value = vm.pop(&addr.clone())?;
            let timestamp = utils::expect_any(
                addr.clone(),
                timestamp_value,
                Some(Error::own_text(
                    addr.clone(),
                    format!("not a timestamp: {timestamp_value:?}"),
                    "check your code.",
                )),
            );
            match (*timestamp).downcast_mut::<DateTime<Local>>() {
                Some(time) => {
                    if should_push {
                        vm.push(Value::Int(time.minute().into()));
                    }
                }
                None => {
                    error!(Error::own_text(
                        addr,
                        format!("invalid timestamp: {timestamp:?}"),
                        "check your code.",
                    ))
                }
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "timestamp@hour",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let timestamp_value = vm.pop(&addr.clone())?;
            let timestamp = utils::expect_any(
                addr.clone(),
                timestamp_value,
                Some(Error::own_text(
                    addr.clone(),
                    format!("not a timestamp: {timestamp_value:?}"),
                    "check your code.",
                )),
            );
            match (*timestamp).downcast_mut::<DateTime<Local>>() {
                Some(time) => {
                    if should_push {
                        vm.push(Value::Int(time.hour().into()));
                    }
                }
                None => {
                    error!(Error::own_text(
                        addr,
                        format!("invalid timestamp: {timestamp:?}"),
                        "check your code.",
                    ))
                }
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "timestamp@day",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let timestamp_value = vm.pop(&addr.clone())?;
            let timestamp = utils::expect_any(
                addr.clone(),
                timestamp_value,
                Some(Error::own_text(
                    addr.clone(),
                    format!("not a timestamp: {timestamp_value:?}"),
                    "check your code.",
                )),
            );
            match (*timestamp).downcast_mut::<DateTime<Local>>() {
                Some(time) => {
                    if should_push {
                        vm.push(Value::Int(time.day().into()));
                    }
                }
                None => {
                    error!(Error::own_text(
                        addr,
                        format!("invalid timestamp: {timestamp:?}"),
                        "check your code.",
                    ))
                }
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "timestamp@year",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let timestamp_value = vm.pop(&addr.clone())?;
            let timestamp = utils::expect_any(
                addr.clone(),
                timestamp_value,
                Some(Error::own_text(
                    addr.clone(),
                    format!("not a timestamp: {timestamp_value:?}"),
                    "check your code.",
                )),
            );
            match (*timestamp).downcast_mut::<DateTime<Local>>() {
                Some(time) => {
                    if should_push {
                        vm.push(Value::Int(time.year().into()));
                    }
                }
                None => {
                    error!(Error::own_text(
                        addr,
                        format!("invalid timestamp: {timestamp:?}"),
                        "check your code.",
                    ))
                }
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "timestamp@month",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let timestamp_value = vm.pop(&addr.clone())?;
            let timestamp = utils::expect_any(
                addr.clone(),
                timestamp_value,
                Some(Error::own_text(
                    addr.clone(),
                    format!("not a timestamp: {timestamp_value:?}"),
                    "check your code.",
                )),
            );
            match (*timestamp).downcast_mut::<DateTime<Local>>() {
                Some(time) => {
                    if should_push {
                        vm.push(Value::Int(time.month().into()));
                    }
                }
                None => {
                    error!(Error::own_text(
                        addr,
                        format!("invalid timestamp: {timestamp:?}"),
                        "check your code.",
                    ))
                }
            }
            Ok(())
        },
    );
    // успех
    Ok(())
}
