// imports
use crate::natives::natives;
use crate::table::Table;
use crate::values::Value;
use crate::vm::VM;
use watt_common::address::Address;
use watt_common::errors::Error;

/// Provides
#[allow(unused_variables)]
pub unsafe fn provide(built_in_address: &Address, vm: &mut VM) -> Result<(), Error> {
    natives::provide(
        vm,
        built_in_address.clone(),
        0,
        "gc@invoke",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            vm.gc_invoke(table);
            if should_push {
                vm.push(Value::Null)
            }
            Ok(())
        },
    );
    Ok(())
}
