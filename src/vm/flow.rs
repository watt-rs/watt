use crate::errors::Error;
use crate::vm::values::Value;

// контрол флоу
pub enum ControlFlow {
    Return(*mut Value),
    Error(Error),
    Continue,
    Break,
}