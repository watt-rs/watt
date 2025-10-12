/// Imports
use crate::typ::Module;
use ecow::EcoString;
use oil_common::rc_ptr::RcPtr;
use std::collections::HashMap;

/// Root ctx
pub struct RootCx {
    /// Analyzed modules
    pub modules: HashMap<EcoString, RcPtr<Module>>,
}
