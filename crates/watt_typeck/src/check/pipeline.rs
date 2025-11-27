use crate::cx::module::ModuleCx;
use crate::typ::typ::Module;
/// Imports
use log::info;
use watt_ast::ast::Declaration;

/// Skip macro
macro_rules! skip {
    () => {{}};
}

/// Implementation
impl<'pkg, 'cx> ModuleCx<'pkg, 'cx> {
    /// Runs pipelined check on the module
    ///
    /// Pipeline stages:
    /// 1. Perform imports.
    /// 2. Early define types by name.
    /// 3. Early define and analyze functions.
    /// 4. Late analyze declarations.
    ///
    /// After this call, the module is fully type-checked.
    pub(crate) fn pipeline(&mut self) -> Module {
        // 1. Performing imports
        info!("Performing imports...");
        for import in self.module.dependencies.clone() {
            self.perform_import(import)
        }

        // 2. Early definitions of types
        info!("Performing early type definitions.");
        for definition in &self.module.declarations {
            match definition {
                Declaration::Type(t) => self.early_define_type(t),
                _ => skip!(),
            }
        }

        // 3. Early functions analysis
        info!("Performing early functions analyse.");
        for definition in &self.module.declarations {
            match definition {
                Declaration::Fn(f) => self.early_define_fn(f),
                _ => skip!(),
            }
        }

        // 4. Late analysis
        info!("Performing late analysis...");
        for definition in self.module.declarations.clone() {
            self.late_analyze_declaration(definition);
        }

        // Pipeline result
        Module {
            source: self.module.source.clone(),
            name: self.module_name.clone(),
            fields: self.resolver.collect(),
        }
    }
}
