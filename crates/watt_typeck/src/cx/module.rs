/// Imports
use crate::{
    cx::package::PackageCx, resolve::resolve::ModuleResolver, typ::Module, unify::EquationsSolver,
};
use ecow::EcoString;
use log::info;
use watt_ast::ast::{self};

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
    pub(crate) solver: EquationsSolver<'cx>,
    /// Last uid
    last_uid: usize,
}

/// Implementation
impl<'pkg, 'cx> ModuleCx<'pkg, 'cx> {
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
            solver: EquationsSolver::new(package),
            last_uid: 0,
        }
    }

    /// Performs analyze of module
    pub fn analyze(&mut self) -> Module {
        // 1. Performing imports
        info!("Performing imports...");
        for import in self.module.dependencies.clone() {
            self.perform_import(import)
        }

        // 2. Early definitions
        info!("Performing early analysis... Stage: early definitions.");
        for definition in &self.module.declarations {
            self.early_define(definition);
        }

        // 3. Early analysys
        info!("Performing early analysis... Stage: early analysis.");
        for definition in &self.module.declarations {
            self.early_analyze(definition)
        }

        // 4. Late analysys
        info!("Performing late analysys...");
        for definition in self.module.declarations.clone() {
            self.late_analyze_declaration(definition);
        }

        Module {
            source: self.module.source.clone(),
            name: self.module_name.clone(),
            fields: self.resolver.collect(),
        }
    }

    /// Generates fresh uid
    pub fn fresh_id(&mut self) -> usize {
        self.last_uid += 1;
        return self.last_uid - 1;
    }
}
