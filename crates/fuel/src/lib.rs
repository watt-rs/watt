#[allow(unsafe_op_in_unsafe_fn)]
pub mod bytecode;
pub(crate) mod call_stack;
pub(crate) mod flow;
pub mod memory;
pub(crate) mod natives;
pub mod values;
pub mod vm;
