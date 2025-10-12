/// Imports
use crate::{cx::package::PackageCx, resolve::resolve::ModuleResolver, typ::Module};
use ecow::EcoString;
use oil_ir::ir::IrModule;

/// Module ctx
pub struct ModuleCx<'pkg, 'cx> {
    /// Current analyzing module info
    pub(crate) module: &'pkg IrModule,
    pub(crate) module_name: &'pkg EcoString,
    /// Resolver
    pub(crate) resolver: ModuleResolver,
    /// Root package context
    pub(crate) package: &'cx PackageCx<'cx>,
}

/// Implementation
impl<'pkg, 'cx> ModuleCx<'pkg, 'cx> {
    /// Creates new module analyzer
    pub fn new(
        module: &'pkg IrModule,
        module_name: &'pkg EcoString,
        package: &'cx PackageCx<'pkg>,
    ) -> Self {
        Self {
            module,
            module_name,
            resolver: ModuleResolver::new(),
            package,
        }
    }

    /// Performs analyze of module
    pub fn analyze(&mut self) -> Module {
        for import in self.module.dependencies.clone() {
            self.perform_import(import)
        }
        for definition in self.module.definitions.clone() {
            self.analyze_declaration(definition)
        }
        Module {
            source: self.module.source.clone(),
            name: self.module_name.clone(),
            fields: self.resolver.collect(),
        }
    }
}
