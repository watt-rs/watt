/// Imports
use ecow::EcoString;
use oil_analyze::untyped_ir::untyped_ir::UntypedModule;
use std::collections::HashMap;

/// Project compiler
pub struct ProjectCompiler {
    /// Untyped modules map
    pub untyped_modules: HashMap<EcoString, UntypedModule>,
}

/// Project compiler implementation
impl ProjectCompiler {
    /// Creates new project compiler
    pub fn new() -> Self {
        Self {
            untyped_modules: HashMap::new(),
        }
    }

    /// Defines untyped module
    pub fn define_untyped(&mut self, name: EcoString, module: UntypedModule) {
        // todo
        self.untyped_modules.insert(name, module);
    }
}
