/// Imports
use crate::{
    cx::package::PackageCx,
    resolve::resolve::ModuleResolver,
    typ::{
        cx::{InferCx, TyCx},
        typ::Module,
    },
};
use ecow::EcoString;
use watt_ast::ast::{self};

/// Module ctx
pub struct ModuleCx<'pkg, 'cx> {
    /// Current analyzing module info
    pub(crate) module: &'pkg ast::Module,
    pub(crate) module_name: &'pkg EcoString,
    /// Resolver
    pub(crate) resolver: ModuleResolver,
    /// Inference context
    pub(crate) icx: InferCx<'cx>,
    /// Root package context
    pub(crate) package: &'cx PackageCx<'cx>,
    /// Last uid
    last_uid: usize,
}

/// Implementation
impl<'pkg, 'cx> ModuleCx<'pkg, 'cx> {
    /// Creates new module analyzer
    pub fn new(
        module: &'pkg ast::Module,
        module_name: &'pkg EcoString,
        tcx: &'cx mut TyCx,
        package: &'cx PackageCx<'pkg>,
    ) -> Self {
        Self {
            module,
            module_name,
            resolver: ModuleResolver::default(),
            icx: InferCx::new(tcx),
            package,
            last_uid: 0,
        }
    }

    /// Performs analyze of module
    pub fn analyze(&mut self) -> Module {
        self.pipeline()
    }

    /// Generates fresh uid
    pub fn fresh_id(&mut self) -> usize {
        self.last_uid += 1;
        self.last_uid - 1
    }
}
