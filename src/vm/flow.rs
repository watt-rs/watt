// импорты
use crate::vm::values::Value;

// контрол флоу
#[derive(Debug)]
pub enum ControlFlow {
    Return(Value),
    Continue,
    Break,
}