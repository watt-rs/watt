/// Imports
use crate::typ::Module;
use ecow::EcoString;
use watt_common::rc_ptr::RcPtr;
use std::collections::HashMap;

/// Root ctx
pub struct RootCx {
    /// Analyzed modules
    pub modules: HashMap<EcoString, RcPtr<Module>>,
}
