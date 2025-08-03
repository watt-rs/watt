// imports
use crate::flow::ControlFlow;
use crate::memory::memory;
use crate::natives::libs::*;
use crate::table::Table;
use crate::values::{Native, Value};
use crate::vm::VM;
use watt_common::address::Address;
use watt_common::errors::Error;

/// Provides builtins
pub unsafe fn provide_builtins(vm: &mut VM) -> Result<(), Error> {
    // builtin address
    let built_in_address: Address = Address::unknown();

    // natives provide
    natives_base::provide(&built_in_address, vm)?;
    natives_io::provide(&built_in_address, vm)?;
    natives_list::provide(&built_in_address, vm)?;
    natives_gc::provide(&built_in_address, vm)?;
    natives_convert::provide(&built_in_address, vm)?;
    natives_time::provide(&built_in_address, vm)?;
    natives_fs::provide(&built_in_address, vm)?;
    natives_system::provide(&built_in_address, vm)?;
    natives_math::provide(&built_in_address, vm)?;
    natives_crypto::provide(&built_in_address, vm)?;
    natives_strings::provide(&built_in_address, vm)?;
    natives_ffi::provide(&built_in_address, vm)?;
    natives_net::provide(&built_in_address, vm)?;

    Ok(())
}

/// Provides single native
pub unsafe fn provide(
    vm: &mut VM,
    addr: Address,
    params_amount: usize,
    name: &'static str,
    native: fn(&mut VM, Address, bool, *mut Table) -> Result<(), ControlFlow>,
) {
    // native value
    let native_fn = Value::Native(memory::alloc_value(Native::new(
        name.to_owned(),
        params_amount,
        native,
        vm.natives_table,
    )));
    // guard native in gc, then register
    vm.gc_guard(native_fn);
    vm.gc_register(native_fn, vm.natives_table);
    // define native
    (*vm.natives_table).define(&addr, name, native_fn);
    // unguard native in gc
    vm.gc_unguard();
}
