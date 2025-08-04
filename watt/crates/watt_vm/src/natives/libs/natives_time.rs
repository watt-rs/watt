// imports
use crate::bytecode::OpcodeValue;
use crate::flow::ControlFlow;
use crate::memory::gc::Gc;
use crate::memory::memory;
use crate::natives::natives;
use crate::natives::utils;
use crate::table::Table;
use crate::values::Value;
use crate::vm::VM;
use chrono::{DateTime, Datelike, Duration, Local, TimeDelta, Timelike};
use watt_common::address::Address;
use watt_common::{error, errors::Error};

/// Gets timestamp from stack
unsafe fn pop_timestamp<'vm>(
    vm: &'vm mut VM,
    addr: &Address,
) -> Result<&'vm mut DateTime<Local>, ControlFlow> {
    // getting a raw timestamp
    let raw_timestamp = utils::expect_any(addr, vm.pop(addr), None);

    if !(*raw_timestamp).is::<DateTime<Local>>() {
        error!(Error::new(
            addr.clone(),
            "internal type in std.time.Timestamp is not a Rust's `chrono::DateTime<Local>`!",
            "please, file an issue at https://github.com/vyacheslavhere/watt"
        ));
    }

    Ok((*raw_timestamp).downcast_mut().unwrap())
}

/// Gets timedelta from stack
unsafe fn pop_timedelta<'vm>(
    vm: &'vm mut VM,
    addr: &Address,
) -> Result<&'vm mut TimeDelta, ControlFlow> {
    // getting a raw timedelta
    let raw_timedelta = utils::expect_any(addr, vm.pop(addr), None);

    if !(*raw_timedelta).is::<TimeDelta>() {
        error!(Error::new(
            addr.clone(),
            "internal type in std.time.Timedelta is not a Rust's `chrono::TimeDelta`!",
            "please, file an issue at https://github.com/vyacheslavhere/watt"
        ));
    }

    Ok((*raw_timedelta).downcast_mut().unwrap())
}

/// Provides
#[allow(unused_variables)]
pub unsafe fn provide(built_in_address: &Address, vm: &mut VM) -> Result<(), Error> {
    natives::provide(
        vm,
        built_in_address.clone(),
        0,
        "time@now",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            if should_push {
                vm.op_push(OpcodeValue::Raw(Value::Any(Gc::new(memory::alloc_value(
                    Local::now(),
                )))))?;
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "timestamp@millis",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let timestamp: &mut DateTime<Local> = pop_timestamp(vm, &addr)?;
            let milis = timestamp.timestamp_millis();
            if should_push {
                vm.push(Value::Int(milis));
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "timestamp@seconds",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let timestamp: &mut DateTime<Local> = pop_timestamp(vm, &addr)?;
            let milis = timestamp.timestamp();
            if should_push {
                vm.push(Value::Int(milis));
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "timestamp@second",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let timestamp: &mut DateTime<Local> = pop_timestamp(vm, &addr)?;
            let second = timestamp.second().into();
            if should_push {
                vm.push(Value::Int(second));
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "timestamp@minute",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let timestamp: &mut DateTime<Local> = pop_timestamp(vm, &addr)?;
            let minute = timestamp.minute().into();
            if should_push {
                vm.push(Value::Int(minute));
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "timestamp@hour",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let timestamp: &mut DateTime<Local> = pop_timestamp(vm, &addr)?;
            let hour = timestamp.hour().into();
            if should_push {
                vm.push(Value::Int(hour));
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "timestamp@day",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let timestamp: &mut DateTime<Local> = pop_timestamp(vm, &addr)?;
            let day = timestamp.day().into();
            if should_push {
                vm.push(Value::Int(day));
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "timestamp@year",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let timestamp: &mut DateTime<Local> = pop_timestamp(vm, &addr)?;
            let year = timestamp.year().into();
            if should_push {
                vm.push(Value::Int(year));
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "timestamp@month",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let timestamp: &mut DateTime<Local> = pop_timestamp(vm, &addr)?;
            let month = timestamp.month().into();
            if should_push {
                vm.push(Value::Int(month));
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "timestamp@weekday",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let timestamp: &mut DateTime<Local> = pop_timestamp(vm, &addr)?;
            let weekday = timestamp.weekday().num_days_from_monday() as i64;
            if should_push {
                vm.push(Value::Int(weekday));
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "timestamp@week",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let timestamp: &mut DateTime<Local> = pop_timestamp(vm, &addr)?;
            let week0 = timestamp.iso_week().week0() as i64;
            if should_push {
                vm.push(Value::Int(week0));
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "timestamp@sub",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let second_timestamp = pop_timestamp(vm, &addr)?.to_owned();
            let first_timestamp = pop_timestamp(vm, &addr)?.to_owned();
            let result = first_timestamp - second_timestamp;
            if should_push {
                vm.op_push(OpcodeValue::Raw(Value::Any(Gc::new(memory::alloc_value(
                    result,
                )))))?;
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "timestamp@gt",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let second_timestamp = pop_timestamp(vm, &addr)?.to_owned();
            let first_timestamp = pop_timestamp(vm, &addr)?.to_owned();
            if should_push {
                vm.op_push(OpcodeValue::Bool(first_timestamp > second_timestamp))?;
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "timestamp@lt",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let second_timestamp = pop_timestamp(vm, &addr)?.to_owned();
            let first_timestamp = pop_timestamp(vm, &addr)?.to_owned();
            if should_push {
                vm.op_push(OpcodeValue::Bool(first_timestamp < second_timestamp))?;
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "timestamp@ge",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let second_timestamp = pop_timestamp(vm, &addr)?.to_owned();
            let first_timestamp = pop_timestamp(vm, &addr)?.to_owned();
            if should_push {
                vm.op_push(OpcodeValue::Bool(first_timestamp >= second_timestamp))?;
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "timestamp@le",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let second_timestamp = pop_timestamp(vm, &addr)?.to_owned();
            let first_timestamp = pop_timestamp(vm, &addr)?.to_owned();
            if should_push {
                vm.op_push(OpcodeValue::Bool(first_timestamp <= second_timestamp))?;
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "timestamp@eq",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let second_timestamp = pop_timestamp(vm, &addr)?.to_owned();
            let first_timestamp = pop_timestamp(vm, &addr)?.to_owned();
            if should_push {
                vm.op_push(OpcodeValue::Bool(first_timestamp == second_timestamp))?;
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "timestamp@add_minutes",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let minutes = utils::expect_int(&addr, vm.pop(&addr));
            let mut timestamp = pop_timestamp(vm, &addr)?.to_owned();
            timestamp += Duration::minutes(minutes);
            if should_push {
                vm.op_push(OpcodeValue::Raw(Value::Any(Gc::new(memory::alloc_value(
                    timestamp,
                )))))?;
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "timestamp@add_seconds",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let seconds = utils::expect_int(&addr, vm.pop(&addr));
            let mut timestamp = pop_timestamp(vm, &addr)?.to_owned();
            timestamp += Duration::seconds(seconds);
            if should_push {
                vm.op_push(OpcodeValue::Raw(Value::Any(Gc::new(memory::alloc_value(
                    timestamp,
                )))))?;
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "timestamp@add_hours",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let hours = utils::expect_int(&addr, vm.pop(&addr));
            let mut timestamp = pop_timestamp(vm, &addr)?.to_owned();
            timestamp += Duration::hours(hours);
            if should_push {
                vm.op_push(OpcodeValue::Raw(Value::Any(Gc::new(memory::alloc_value(
                    timestamp,
                )))))?;
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "timestamp@add_weeks",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let weeks = utils::expect_int(&addr, vm.pop(&addr));
            let mut timestamp = pop_timestamp(vm, &addr)?.to_owned();
            timestamp += Duration::weeks(weeks);
            if should_push {
                vm.op_push(OpcodeValue::Raw(Value::Any(Gc::new(memory::alloc_value(
                    timestamp,
                )))))?;
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "timestamp@add_millis",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let millis = utils::expect_int(&addr, vm.pop(&addr));
            let mut timestamp = pop_timestamp(vm, &addr)?.to_owned();
            timestamp += Duration::milliseconds(millis);
            if should_push {
                vm.op_push(OpcodeValue::Raw(Value::Any(Gc::new(memory::alloc_value(
                    timestamp,
                )))))?;
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "timestamp@add_days",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let days = utils::expect_int(&addr, vm.pop(&addr));
            let mut timestamp = pop_timestamp(vm, &addr)?.to_owned();
            timestamp += Duration::days(days);
            if should_push {
                vm.op_push(OpcodeValue::Raw(Value::Any(Gc::new(memory::alloc_value(
                    timestamp,
                )))))?;
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "timestamp@add_delta",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let timedelta = pop_timedelta(vm, &addr)?.to_owned();
            let mut timestamp = pop_timestamp(vm, &addr)?.to_owned();
            timestamp += timedelta;
            if should_push {
                vm.op_push(OpcodeValue::Raw(Value::Any(Gc::new(memory::alloc_value(
                    timestamp,
                )))))?;
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "timedelta@millis",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let timedelta = pop_timedelta(vm, &addr)?.to_owned();
            if should_push {
                vm.op_push(OpcodeValue::Raw(Value::Int(timedelta.num_milliseconds())))?;
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "timedelta@seconds",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let timedelta = pop_timedelta(vm, &addr)?.to_owned();
            if should_push {
                vm.op_push(OpcodeValue::Raw(Value::Int(timedelta.num_seconds())))?;
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "timedelta@minutes",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let timedelta = pop_timedelta(vm, &addr)?.to_owned();
            if should_push {
                vm.op_push(OpcodeValue::Raw(Value::Int(timedelta.num_minutes())))?;
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "timedelta@hours",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let timedelta = pop_timedelta(vm, &addr)?.to_owned();
            if should_push {
                vm.op_push(OpcodeValue::Raw(Value::Int(timedelta.num_hours())))?;
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "timedelta@days",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let timedelta = pop_timedelta(vm, &addr)?.to_owned();
            if should_push {
                vm.op_push(OpcodeValue::Raw(Value::Int(timedelta.num_days())))?;
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "timedelta@weeks",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let timedelta = pop_timedelta(vm, &addr)?.to_owned();
            if should_push {
                vm.op_push(OpcodeValue::Raw(Value::Int(timedelta.num_weeks())))?;
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "timedelta@add",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let second_timedelta = pop_timedelta(vm, &addr)?.to_owned();
            let first_timedelta = pop_timedelta(vm, &addr)?.to_owned();
            if should_push {
                vm.op_push(OpcodeValue::Raw(Value::Any(Gc::new(memory::alloc_value(
                    first_timedelta + second_timedelta,
                )))))?;
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "timedelta@sub",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let second_timedelta = pop_timedelta(vm, &addr)?.to_owned();
            let first_timedelta = pop_timedelta(vm, &addr)?.to_owned();
            if should_push {
                vm.op_push(OpcodeValue::Raw(Value::Any(Gc::new(memory::alloc_value(
                    first_timedelta - second_timedelta,
                )))))?;
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "timedelta@gt",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let second_timedelta = pop_timedelta(vm, &addr)?.to_owned();
            let first_timedelta = pop_timedelta(vm, &addr)?.to_owned();
            if should_push {
                vm.op_push(OpcodeValue::Raw(Value::Any(Gc::new(memory::alloc_value(
                    first_timedelta > second_timedelta,
                )))))?;
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "timedelta@lt",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let second_timedelta = pop_timedelta(vm, &addr)?.to_owned();
            let first_timedelta = pop_timedelta(vm, &addr)?.to_owned();
            if should_push {
                vm.op_push(OpcodeValue::Raw(Value::Any(Gc::new(memory::alloc_value(
                    first_timedelta < second_timedelta,
                )))))?;
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "timedelta@ge",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let second_timedelta = pop_timedelta(vm, &addr)?.to_owned();
            let first_timedelta = pop_timedelta(vm, &addr)?.to_owned();
            if should_push {
                vm.op_push(OpcodeValue::Raw(Value::Any(Gc::new(memory::alloc_value(
                    first_timedelta >= second_timedelta,
                )))))?;
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "timedelta@le",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let second_timedelta = pop_timedelta(vm, &addr)?.to_owned();
            let first_timedelta = pop_timedelta(vm, &addr)?.to_owned();
            if should_push {
                vm.op_push(OpcodeValue::Raw(Value::Any(Gc::new(memory::alloc_value(
                    first_timedelta <= second_timedelta,
                )))))?;
            }
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "timedelta@new",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let seconds = utils::expect_int(&addr, vm.pop(&addr));
            if should_push {
                vm.op_push(OpcodeValue::Raw(Value::Any(Gc::new(memory::alloc_value(
                    Duration::new(seconds, 0).unwrap(),
                )))))?;
            }
            Ok(())
        },
    );
    // успех
    Ok(())
}
