/// Imports
use crate::typ::{Function, Typ};
use std::rc::Rc;

/// Call result
#[allow(clippy::enum_variant_names)]
pub enum CallResult {
    FromFunction(Typ, Rc<Function>),
    FromType(Typ),
    FromEnum(Typ),
    FromDyn,
}
