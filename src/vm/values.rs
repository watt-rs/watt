use std::sync::{Arc, Mutex};

// value
#[derive!(Eq, PartialEq, Hash, Debug)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Bool(bool),
    String(String)
}

// sharable value
pub type SharedValue = Arc<Mutex<Value>>;