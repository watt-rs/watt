/// Imports
use ecow::EcoString;
use oil_ir::ir::IrModule;
use std::collections::HashMap;

/// Project compiler
pub struct ProjectCompiler {
    /// Completed modules map
    pub modules: HashMap<EcoString, IrModule>,
}

/// Project compiler implementation
impl ProjectCompiler {
    /// Creates new project compiler
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }
    /// Defines module
    pub fn define_module(&mut self, name: EcoString, module: IrModule) {
        self.modules.insert(name, module);
    }
}
