/// Imports
use crate::{
    cx::package::PackageCx, resolve::resolve::ModuleResolver, typ::Module, unify::EquationsSolver,
};
use ecow::EcoString;
use watt_ast::ast;

/// Module ctx
pub struct ModuleCx<'pkg, 'cx> {
    /// Current analyzing module info
    pub(crate) module: &'pkg ast::Module,
    pub(crate) module_name: &'pkg EcoString,
    /// Resolver
    pub(crate) resolver: ModuleResolver,
    /// Root package context
    pub(crate) package: &'cx PackageCx<'cx>,
    /// Equations solver
    pub(crate) solver: EquationsSolver<'pkg>,
}

/// Implementation
impl<'pkg, 'cx> ModuleCx<'pkg, 'cx>
where
    'pkg: 'cx,
    'cx: 'pkg,
{
    /// Creates new module analyzer
    pub fn new(
        module: &'pkg ast::Module,
        module_name: &'pkg EcoString,
        package: &'cx PackageCx<'pkg>,
    ) -> Self {
        Self {
            module,
            module_name,
            resolver: ModuleResolver::new(),
            package,
            solver: EquationsSolver::new(&module.source),
        }
    }

    /// Performs analyze of module
    pub fn analyze(&mut self) -> Module {
        for import in self.module.dependencies.clone() {
            self.perform_import(import)
        }
        for definition in self.module.declarations.clone() {
            self.analyze_declaration(definition)
        }
        Module {
            source: self.module.source.clone(),
            name: self.module_name.clone(),
            fields: self.resolver.collect(),
        }
    }
}
