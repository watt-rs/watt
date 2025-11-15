/// Imports
use crate::typ::typ::Module;
use ecow::EcoString;
use std::collections::HashMap;
use watt_common::rc_ptr::RcPtr;

/// Root ctx
pub struct RootCx {
    /// Analyzed modules
    pub modules: HashMap<EcoString, RcPtr<Module>>,
}
