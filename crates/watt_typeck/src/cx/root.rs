/// Imports
use crate::typ::typ::Module;
use id_arena::{Arena, Id};

/// Root ctx
#[derive(Default)]
pub struct RootCx {
    /// Analyzed modules
    pub modules: Arena<Module>,
}

/// Implementation
impl RootCx {
    /// Retrieves module by id
    pub fn module(&self, id: Id<Module>) -> &Module {
        self.modules.get(id).expect("invalid Module id")
    }

    /// Queries module by name
    pub fn query_module(&self, name: &str) -> Option<Id<Module>> {
        for (id, module) in &self.modules {
            if module.name == name {
                return Some(id);
            }
        }
        None
    }

    /// Inserts new module
    pub fn insert_module(&mut self, module: Module) -> Id<Module> {
        self.modules.alloc(module)
    }
}
