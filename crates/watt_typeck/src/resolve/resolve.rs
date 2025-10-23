/// Imports
use crate::{
    errors::TypeckError,
    resolve::{
        res::Res,
        rib::{Rib, RibKind, RibsStack},
    },
    typ::{CustomType, Module, Typ, Type, WithPublicity},
};
use ecow::EcoString;
use miette::NamedSource;
use std::{cell::RefCell, collections::HashMap, fmt::Debug, sync::Arc};
use watt_common::{address::Address, bail, rc_ptr::RcPtr};

/// Definition
pub enum Def {
    /// Module definition
    Module(ModDef),
    /// Local definition
    Local(Typ),
}

/// Module definition
///
/// CustomType, Variable
/// definitions for module
///
#[derive(Clone)]
pub enum ModDef {
    /// Custom type
    CustomType(WithPublicity<CustomType>),
    /// Variable
    Variable(WithPublicity<Typ>),
}

/// Debug implementation
impl Debug for ModDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModDef::CustomType(ty) => write!(f, "ModDef({ty:?})"),
            ModDef::Variable(var) => write!(f, "ModDef({var:?})"),
        }
    }
}

/// Module resolver
///
/// * rib_stack - stack of ribs for nested scopes
/// * definitions - module definitions
pub struct ModuleResolver {
    /// Ribs stack of module
    ribs_stack: RibsStack,
    /// Module definitions
    mod_defs: HashMap<EcoString, ModDef>,
    /// Imported modules
    pub imported_modules: HashMap<EcoString, RcPtr<Module>>,
    /// Imported modules
    pub imported_defs: HashMap<EcoString, ModDef>,
}

/// Implementation
impl ModuleResolver {
    /// Creates new module resolver
    pub fn new() -> Self {
        Self {
            ribs_stack: RibsStack::new(),
            mod_defs: HashMap::new(),
            imported_modules: HashMap::new(),
            imported_defs: HashMap::new(),
        }
    }

    /// Creates definition, if no definition
    /// with same name is already defined.
    pub fn define(
        &mut self,
        named_source: &NamedSource<Arc<String>>,
        address: &Address,
        name: &EcoString,
        def: Def,
    ) {
        // Checking def
        match def {
            // If module
            Def::Module(mod_def) => {
                // Checking already defined
                match self.mod_defs.get(name) {
                    // If already defined
                    Some(_) => match mod_def {
                        // Custom type
                        ModDef::CustomType(_) => {
                            bail!(TypeckError::TypeIsAlreadyDefined {
                                src: named_source.clone(),
                                span: address.span.clone().into(),
                                t: name.clone()
                            })
                        }
                        // Variable
                        ModDef::Variable(_) => {
                            bail!(TypeckError::VariableIsAlreadyDefined {
                                src: named_source.clone(),
                                span: address.span.clone().into(),
                            })
                        }
                    },
                    // If not
                    None => {
                        self.mod_defs.insert(name.clone(), mod_def);
                    }
                }
            }
            // If local
            Def::Local(local_def) => {
                self.ribs_stack
                    .define(named_source, address, name, local_def);
            }
        }
    }

    /// Defines local variable.
    /// If definition exists, checks types equality.
    pub fn redefine_local(
        &mut self,
        named_source: &NamedSource<Arc<String>>,
        address: &Address,
        name: &EcoString,
        typ: Typ,
    ) {
        self.ribs_stack.redefine(named_source, address, name, typ);
    }

    /// Resolves up a value
    /// Raises error if variable is not found.
    pub fn resolve(
        &self,
        named_source: &NamedSource<Arc<String>>,
        address: &Address,
        name: &EcoString,
    ) -> Res {
        // Checking existence in ribs
        match self.ribs_stack.lookup(name) {
            Some(typ) => Res::Value(typ),
            None => match self.mod_defs.get(name) {
                // Checking existence in module definitions
                Some(typ) => match typ {
                    ModDef::CustomType(ty) => Res::Custom(ty.value.clone()),
                    ModDef::Variable(var) => Res::Value(var.value.clone()),
                },
                None => match self.imported_defs.get(name) {
                    // Checking existence in imported defs
                    Some(typ) => match typ {
                        ModDef::CustomType(ty) => Res::Custom(ty.value.clone()),
                        ModDef::Variable(var) => Res::Value(var.value.clone()),
                    },
                    None => match self.imported_modules.get(name) {
                        // Checking existence in modules
                        Some(_) => Res::Module(name.clone()),
                        None => bail!(TypeckError::CouldNotResolve {
                            src: named_source.clone(),
                            span: address.clone().span.into(),
                            name: name.clone()
                        }),
                    },
                },
            },
        }
    }

    /// Resolves up a type
    /// Raises error if variable is not found.
    pub fn resolve_type(
        &self,
        name: &EcoString,
        named_source: &NamedSource<Arc<String>>,
        address: &Address,
    ) -> CustomType {
        // Checking existence in module definitions
        match self.mod_defs.get(name) {
            Some(typ) => match typ {
                ModDef::CustomType(ty) => ty.value.clone(),
                ModDef::Variable(_) => bail!(TypeckError::CouldNotUseValueAsType {
                    src: named_source.clone(),
                    span: address.clone().span.into(),
                    v: name.clone()
                }),
            },
            None => match self.imported_defs.get(name) {
                // Checking existence in imported defs
                Some(typ) => match typ {
                    ModDef::CustomType(ty) => ty.value.clone(),
                    ModDef::Variable(_) => bail!(TypeckError::CouldNotUseValueAsType {
                        src: named_source.clone(),
                        span: address.clone().span.into(),
                        v: name.clone()
                    }),
                },
                None => bail!(TypeckError::TypeIsNotDefined {
                    src: named_source.clone(),
                    span: address.clone().span.into(),
                    t: name.clone()
                }),
            },
        }
    }

    /// Resolves up a module
    pub fn resolve_module(&self, name: &EcoString) -> &Module {
        // Checking existence in module definitions
        match self.imported_modules.get(name) {
            Some(m) => m,
            None => bail!(TypeckError::ModuleIsNotDefined { m: name.clone() }),
        }
    }

    /// Contains type rib
    pub fn contains_type_rib(&self) -> Option<&RcPtr<RefCell<Type>>> {
        self.ribs_stack.contains_type()
    }

    /// Contains rib with specifix kind
    pub fn contains_rib(&self, kind: RibKind) -> bool {
        self.ribs_stack.contains_rib(kind)
    }

    /// Pushes rib
    pub fn push_rib(&mut self, kind: RibKind) {
        self.ribs_stack.push(kind);
    }

    /// Pops rib
    pub fn pop_rib(&mut self) -> Option<Rib> {
        self.ribs_stack.pop()
    }

    /// Collects fields from resolver
    pub fn collect(&mut self) -> HashMap<EcoString, ModDef> {
        self.mod_defs.drain().collect()
    }

    /// Imports module as name
    pub fn import_as(
        &mut self,
        named_source: &NamedSource<Arc<String>>,
        address: &Address,
        name: EcoString,
        module: RcPtr<Module>,
    ) {
        match self.imported_modules.get(&name) {
            Some(module) => bail!(TypeckError::ModuleIsAlreadyImportedAs {
                src: named_source.clone(),
                span: address.span.clone().into(),
                m: module.name.clone(),
                name: name.clone()
            }),
            None => self.imported_modules.insert(name, module),
        };
    }

    /// Imports names from module
    pub fn import_for(
        &mut self,
        named_source: &NamedSource<Arc<String>>,
        address: &Address,
        names: Vec<EcoString>,
        module: RcPtr<Module>,
    ) {
        // Importing names
        for name in names {
            // Checking name existence
            match module.fields.get(&name) {
                Some(def) => match self.imported_defs.get(&name) {
                    // Checking name is already imported from other module
                    Some(already) => bail!(TypeckError::DefIsAlreadyImported {
                        src: named_source.clone(),
                        span: address.span.clone().into(),
                        name: name.clone(),
                        def: already.clone()
                    }),
                    None => {
                        self.imported_defs.insert(name, def.clone());
                    }
                },
                None => {
                    bail!(TypeckError::ModuleFieldIsNotDefined {
                        src: named_source.clone(),
                        span: address.span.clone().into(),
                        m: module.name.clone(),
                        field: name
                    })
                }
            }
        }
    }
}
