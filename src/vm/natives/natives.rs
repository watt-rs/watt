// imports
use crate::errors::errors::Error;
use crate::lexer::address::Address;
use crate::vm::flow::ControlFlow;
use crate::vm::memory::memory;
use crate::vm::natives::libs::*;
use crate::vm::table::Table;
use crate::vm::values::{Native, Symbol, Value};
use crate::vm::vm::VM;

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
        Symbol::by_name(name.to_owned()),
        params_amount,
        native,
    )));
    // guard native in gc, then register
    vm.gc_guard(native_fn);
    vm.gc_register(native_fn, vm.globals);
    // define native
    (*vm.natives).define(&addr, name, native_fn);
    // unguard native in gc
    vm.gc_unguard();
}
