/// Imports
use crate::typ::{Function, Typ};
use oil_common::rc_ptr::RcPtr;

/// Call result
#[allow(clippy::enum_variant_names)]
pub enum CallResult {
    FromFunction(Typ, RcPtr<Function>),
    FromType(Typ),
    FromEnum(Typ),
    FromDyn,
}
