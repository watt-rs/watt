use std::collections::BTreeMap;
use crate::vm::values::Value;

#[derive(Clone, Debug)]
pub struct Table {
    fields: BTreeMap<String, Value>,
}