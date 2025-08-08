// imports
use crate::flow::ControlFlow;
use crate::memory::gc::Gc;
use crate::natives::libraries::{natives_base, natives_io};
use crate::table::Table;
use crate::values::{Native, Value};
use crate::vm::VM;
use oil_common::address::Address;
use oil_common::errors::Error;

/// Provides builtins
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe fn provide_natives(vm: &mut VM) -> Result<(), Error> {
    // builtin address
    let built_in_address: Address = Address::unknown();

    // natives provide
    natives_base::provide(&built_in_address, vm)?;
    natives_io::provide(&built_in_address, vm)?;

    Ok(())
}

/// Provides single native
pub unsafe fn provide(
    vm: &mut VM,
    addr: Address,
    params_amount: usize,
    name: &'static str,
    native: fn(&mut VM, Address, bool, Gc<Table>) -> Result<(), ControlFlow>,
) {
    // native value
    let native_fn = Value::Native(Gc::new(Native::new(name.to_owned(), params_amount, native)));
    // define native
    (*vm.natives_table).define(&addr, name, native_fn);
}
