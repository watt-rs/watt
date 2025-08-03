// fixed 24 edition warnings
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(clippy::module_inception)]
#![allow(clippy::too_many_arguments)]
#![allow(dangerous_implicit_autorefs)]

// modules
pub mod bytecode;
pub(crate) mod flow;
pub mod memory;
pub(crate) mod natives;
pub(crate) mod table;
pub mod values;
pub mod vm;
