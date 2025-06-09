use crate::errors::Error;
use crate::vm::values::Value;

// контрол флоу
#[derive(Debug)]
pub enum ControlFlow {
    Return(Value),
    Error(Error),
    Continue,
    Break,
}