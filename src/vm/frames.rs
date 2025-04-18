use std::collections::BTreeMap;
use crate::vm::values::Value;

pub struct Frame {
    map: BTreeMap<String, Value>,
    root: Option<Frame>,
    closure: Option<Frame>
}

impl Frame {
    pub fn new() -> Frame {
        Frame {
            map: BTreeMap::new(),
            root: Option::None,
            closure: Option::None
        }
    }
}