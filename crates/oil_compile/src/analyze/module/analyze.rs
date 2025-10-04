/// Imports
use crate::analyze::{
    errors::AnalyzeError,
    rc_ptr::RcPtr,
    resolve::ModuleResolver,
    typ::{Function, Module, PreludeType, Typ},
};
use ecow::EcoString;
use oil_common::{address::Address, bail};
use oil_ir::ir::IrModule;
use std::collections::HashMap;

/// Call result
pub enum CallResult {
    FromFunction(Typ, RcPtr<Function>),
    FromType(Typ),
    FromEnum(Typ),
    FromDyn,
}

/// Module analyzer
pub struct ModuleAnalyzer<'pkg> {
    /// Current analyzing module info
    pub(crate) module: &'pkg IrModule,
    pub(crate) module_name: &'pkg EcoString,
    /// Resolver
    pub(crate) resolver: ModuleResolver,
    /// Modules available to import
    pub(crate) modules: &'pkg HashMap<EcoString, RcPtr<Module>>,
}

/// Implementation
impl<'pkg> ModuleAnalyzer<'pkg> {
    /// Creates new module analyzer
    pub fn new(
        module: &'pkg IrModule,
        module_name: &'pkg EcoString,
        modules: &'pkg HashMap<EcoString, RcPtr<Module>>,
    ) -> Self {
        Self {
            module,
            module_name,
            resolver: ModuleResolver::new(),
            modules,
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

    /// Unifies two of types, raises error,
    /// if types can't be unified
    pub fn unify(&mut self, location: &Address, t1: &Typ, t2: &Typ) -> Typ {
        if t1 != t2 {
            match (t1, t2) {
                (Typ::Prelude(a), Typ::Prelude(b)) => match (a, b) {
                    (PreludeType::Int, PreludeType::Float) => Typ::Prelude(PreludeType::Float),
                    (PreludeType::Float, PreludeType::Int) => Typ::Prelude(PreludeType::Float),
                    _ => bail!(AnalyzeError::CouldNotUnify {
                        src: self.module.source.clone(),
                        span: location.span.clone().into(),
                        t1: t1.clone(),
                        t2: t2.clone()
                    }),
                },
                (Typ::Dyn, _) | (_, Typ::Dyn) => Typ::Dyn,
                _ => bail!(AnalyzeError::CouldNotUnify {
                    src: self.module.source.clone(),
                    span: location.span.clone().into(),
                    t1: t1.clone(),
                    t2: t2.clone()
                }),
            }
        } else {
            return t1.clone();
        }
    }
}
