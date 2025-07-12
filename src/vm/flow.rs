// imports
use crate::vm::values::Value;

/// ControlFlow structure
/// 
/// used to propagate return, 
/// continue and break in vm
/// 
#[derive(Debug)]
pub enum ControlFlow {
    Return(Value),
    Continue,
    Break,
}