/// Imports
use crate::{
    errors::TypeckError, pretty::Pretty, resolve::rib::{Rib, RibsStack}, typ::{
        cx::InferCx, def::{ModuleDef, TypeDef}, res::Res, typ::{GenericArgs, Module, Typ}
    }
};
use ecow::EcoString;
use std::collections::HashMap;
use tracing::instrument;
use watt_common::{address::Address, bail, rc_ptr::RcPtr};

/// Resolves names and types within a module.
///
/// `ModuleResolver` is responsible for managing module-level scope, tracking
/// definitions, and handling imports. It is used during type checking and
/// name resolution to determine what each identifier refers to.
///
/// # Fields
///
/// - `ribs_stack: RibsStack`
///   A stack of "ribs", representing nested scopes inside the module.
///   Each rib holds bindings for a single lexical scope. The stack structure
///   allows proper shadowing and scope resolution.
///
/// - `mod_defs: HashMap<EcoString, ModuleDef>`
///   The definitions present directly in the module, keyed by their names.
///   This includes user-defined types (`Type`) and constants (`Const`).
///
/// - `imported_modules: HashMap<EcoString, RcPtr<Module>>`
///   Modules that have been imported into the current module.
///   Allows resolution of identifiers that are qualified with module paths.
///
/// - `imported_defs: HashMap<EcoString, ModuleDef>`
///   Definitions imported from other modules, keyed by their local names.
///   Enables access to external types and constants without fully qualifying them.
///
#[derive(Default, Debug)]
pub struct ModuleResolver {
    /// Ribs stack of module
    ribs_stack: RibsStack,
    /// Module definitions
    module_defs: HashMap<EcoString, ModuleDef>,
    /// Imported modules
    pub imported_modules: HashMap<EcoString, RcPtr<Module>>,
    /// Imported definitions
    pub imported_defs: HashMap<EcoString, ModuleDef>,
}

/// Implementation
impl ModuleResolver {
    /// Defines a module-level item (type or constant) if it is not already defined.
    ///
    /// This method inserts a new definition into the module's namespace. It performs
    /// checks to ensure that no conflicting definition exists unless `redefine` is true.
    ///
    /// # Parameters
    ///
    /// - `address: &Address`
    ///   The source location of the definition, used for error reporting.
    ///
    /// - `name: &EcoString`
    ///   The identifier name for the definition.
    ///
    /// - `def: ModuleDef`
    ///   The definition to insert (type or constant).
    ///
    /// - `redefine: bool`
    ///   Allows overwriting an existing definition. This is used during the
    ///   **late analysis pass** to replace temporary definitions created
    ///   during the **early analysis pass**.
    ///
    /// # Behavior
    ///
    /// - If `redefine` is `true`, the new definition always replaces any existing one.
    /// - If `redefine` is `false`:
    ///   - If a type with the same name exists, a `TypeckError::TypeIsAlreadyDefined` is raised.
    ///   - If a constant with the same name exists, a `TypeckError::VariableIsAlreadyDefined` is raised.
    ///   - Otherwise, the new definition is inserted successfully.
    ///
    /// # Important
    ///
    /// - This method ensures that the module maintains a consistent namespace.
    /// - The `redefine` flag is essential for the two-phase analysis process,
    ///   where early passes may create temporary definitions that need to be
    ///   replaced in the late analysis pass.
    ///
    pub fn define_module(
        &mut self,
        address: &Address,
        name: &EcoString,
        def: ModuleDef,
        redefine: bool,
    ) {
        if redefine {
            self.module_defs.insert(name.clone(), def);
        } else {
            match self.module_defs.get(name) {
                Some(found) => match found {
                    ModuleDef::Type(_) => {
                        bail!(TypeckError::TypeIsAlreadyDefined {
                            src: address.source.clone(),
                            span: address.span.clone().into(),
                            t: name.clone()
                        })
                    }
                    ModuleDef::Const(_) | ModuleDef::Function(_) => {
                        bail!(TypeckError::VariableIsAlreadyDefined {
                            src: address.source.clone(),
                            span: address.span.clone().into(),
                            name: name.clone()
                        })
                    }
                },
                None => {
                    self.module_defs.insert(name.clone(), def);
                }
            }
        }
    }

    /// Defines a local-level item (local variable) if it is not already defined.
    ///
    /// This method inserts a new definition into the last rib's scope. It performs
    /// checks to ensure that no conflicting definition exists unless `redefine` is true.
    ///
    /// # Parameters
    ///
    /// - `address: &Address`
    ///   The source location of the definition, used for error reporting.
    ///
    /// - `name: &EcoString`
    ///   The identifier name for the definition.
    ///
    /// - `def: ModuleDef`
    ///   The definition to insert (type or constant).
    ///
    /// - `redefine: bool`
    ///   Allows overwriting an existing definition. This is used during the
    ///   **late analysis pass** to replace temporary definitions created
    ///   during the **early analysis pass**.
    ///
    /// # Behavior
    ///
    /// - If `redefine` is `true`, the new definition always replaces any existing one.
    /// - If `redefine` is `false`:
    ///   - If a variable with the same name exists, a `TypeckError::VariableIsAlreadyDefined` is raised.
    ///   - Otherwise, the new definition is inserted successfully.
    ///
    /// # Important
    ///
    /// - This method ensures that the rib maintains a consistent scope.
    /// - The `redefine` flag is essential for temp variables.
    ///
    pub fn define_local(&mut self, address: &Address, name: &EcoString, typ: Typ, redefine: bool) {
        self.ribs_stack.define(address, name, typ, redefine);
    }

    /// Resolves an identifier to its corresponding value, type, or module.
    ///
    /// This method looks up the given `name` in the current module's namespace
    /// and imported modules/definitions. It follows a structured lookup order
    /// to ensure correct scoping and resolution of identifiers. If the identifier
    /// cannot be found, it raises a `TypeckError::CouldNotResolve`.
    ///
    /// # Parameters
    ///
    /// - `address: &Address`
    ///   The source code location of the identifier being resolved, used for
    ///   error reporting.
    ///
    /// - `name: &EcoString`
    ///   The identifier to resolve.
    ///
    /// # Resolution Flow
    ///
    /// 1. **Ribs stack lookup (local and nested scopes)**
    ///    The method first searches the `ribs_stack`, which represents nested
    ///    lexical scopes. If a binding is found here, it is returned as
    ///    `Res::Value(typ)`. This ensures that local variables shadow module-level
    ///    definitions.
    ///
    /// 2. **Module definitions lookup**
    ///    If the identifier is not found in the local ribs, the resolver checks
    ///    `module_defs`, which contains definitions directly declared in the module:
    ///    - `ModuleDef::Type` -> returned as `Res::Custom(TypeDef)`
    ///    - `ModuleDef::Const` -> returned as `Res::Value(Typ)`
    ///
    /// 3. **Imported definitions lookup**
    ///    If the identifier is not present in local module definitions, the resolver
    ///    checks `imported_defs`, which contains definitions imported from other
    ///    modules. The resolution behaves similarly to module definitions:
    ///    - `ModuleDef::Type` -> `Res::Custom(TypeDef)`
    ///    - `ModuleDef::Const` -> `Res::Value(Typ)`
    ///
    /// 4. **Imported modules lookup**
    ///    If the identifier is not found in definitions, the resolver checks
    ///    `imported_modules`. If found, the identifier resolves to a module:
    ///    - returned as `Res::Module(name.clone())`
    ///
    /// 5. **Error if not found**
    ///    If the identifier cannot be found in any of the above cases, the
    ///    resolver raises a `TypeckError::CouldNotResolve` with the given source
    ///    location and the unresolved name.
    ///
    /// # Returns
    ///
    /// A `Res` enum indicating the resolved entity:
    /// - `Res::Module(EcoString)` -> a module
    /// - `Res::Custom(TypeDef)` -> a user-defined type
    /// - `Res::Value(Typ)` -> a value
    /// - `Res::Const(Typ)` -> a constant value
    ///
    /// `Res::Variant(Rc<Enum>, EnumVariant)` will be never returned
    ///
    pub fn resolve(&self, address: &Address, name: &EcoString) -> Res {
        // Checking existence in ribs
        match self.ribs_stack.lookup(name) {
            Some(typ) => Res::Value(typ),
            None => match self.module_defs.get(name) {
                // Checking existence in module definitions
                Some(typ) => match typ {
                    ModuleDef::Type(ty) => Res::Custom(ty.value.clone()),
                    ModuleDef::Const(ty) => Res::Const(ty.value.clone()),
                    ModuleDef::Function(ty) => {
                        Res::Value(Typ::Function(ty.value.clone(), GenericArgs::default()))
                    }
                },
                None => match self.imported_defs.get(name) {
                    // Checking existence in imported defs
                    Some(typ) => match typ {
                        ModuleDef::Type(ty) => Res::Custom(ty.value.clone()),
                        ModuleDef::Const(ty) => Res::Const(ty.value.clone()),
                        ModuleDef::Function(ty) => {
                            Res::Value(Typ::Function(ty.value.clone(), GenericArgs::default()))
                        }
                    },
                    None => match self.imported_modules.get(name) {
                        // Checking existence in modules
                        Some(_) => Res::Module(name.clone()),
                        None => bail!(TypeckError::CouldNotResolve {
                            src: address.source.clone(),
                            span: address.clone().span.into(),
                            name: name.clone()
                        }),
                    },
                },
            },
        }
    }

    /// Resolves an identifier to its corresponding type.
    ///
    /// This method looks up the given `name` in the current module's namespace
    /// and imported definitions. It follows a structured lookup order
    /// to ensure correct scoping and resolution of identifiers.
    ///
    /// # Parameters
    ///
    /// - `address: &Address`
    ///   The source code location of the identifier being resolved, used for
    ///   error reporting.
    ///
    /// - `name: &EcoString`
    ///   The type name to resolve.
    ///
    /// # Resolution Flow
    ///
    /// 1. **Module definitions**
    ///    The method first checks `module_defs`
    ///    for the type existence (`TypeDef`).
    ///
    /// 2. **Imported definitions lookup**
    ///    If the identifier is not present in module definitions, the resolver
    ///    checks `imported_defs` for the type existence (`TypeDef`), which contains
    ///    definitions imported from other modules.
    ///
    /// # Errors
    ///
    /// - Raises `TypeckError::TypeIsNotDefined` if the type cannot be resolved.
    /// - Raises `TypeckError::CouldNotUseValueAsType` if the const shadows the type name.
    ///
    pub fn resolve_type(&self, address: &Address, name: &EcoString) -> TypeDef {
        // Checking existence in module definitions
        match self.module_defs.get(name) {
            Some(typ) => match typ {
                ModuleDef::Type(ty) => ty.value.clone(),
                ModuleDef::Const(_) | ModuleDef::Function(_) => {
                    bail!(TypeckError::CouldNotUseValueAsType {
                        src: address.source.clone(),
                        span: address.clone().span.into(),
                        v: name.clone()
                    })
                }
            },
            None => match self.imported_defs.get(name) {
                // Checking existence in imported defs
                Some(typ) => match typ {
                    ModuleDef::Type(ty) => ty.value.clone(),
                    ModuleDef::Const(_) | ModuleDef::Function(_) => {
                        bail!(TypeckError::CouldNotUseValueAsType {
                            src: address.source.clone(),
                            span: address.clone().span.into(),
                            v: name.clone()
                        })
                    }
                },
                None => bail!(TypeckError::TypeIsNotDefined {
                    src: address.source.clone(),
                    span: address.clone().span.into(),
                    t: name.clone()
                }),
            },
        }
    }

    /// Resolves a module by its name in the imported modules.
    ///
    /// # Parameters
    /// - `name: &EcoString` — The name of the module to resolve.
    ///
    /// # Returns
    /// - A reference to the `Module` if it exists in `imported_modules`.
    ///
    /// # Errors
    /// - Raises `TypeckError::ModuleIsNotDefined` if the module is not imported.
    ///
    pub fn resolve_module(&self, name: &EcoString) -> &Module {
        match self.imported_modules.get(name) {
            Some(m) => m,
            None => bail!(TypeckError::ModuleIsNotDefined { m: name.clone() }),
        }
    }

    /// Pushes a new rib onto the ribs stack.
    ///
    /// Ribs represent lexical scopes (e.g., for function bodies or blocks).
    /// Pushing a rib creates a new nested scope for variable bindings.
    ///
    pub fn push_rib(&mut self) {
        self.ribs_stack.push();
    }

    /// Pops the top rib from the ribs stack.
    ///
    /// Returns the popped `Rib` if it exists. Popping a rib exits
    /// the current scope and removes all bindings defined in that scope.
    ///
    pub fn pop_rib(&mut self) -> Option<Rib> {
        self.ribs_stack.pop()
    }

    /// Collects and drains all module definitions.
    ///
    /// This method removes all current definitions from `module_defs` and
    /// returns them as a `HashMap`. Useful for exporting definitions after
    /// analysis or for module construction.
    ///
    /// # Returns
    /// A `HashMap<EcoString, ModuleDef>` containing all collected definitions.
    ///
    pub fn collect(&mut self) -> HashMap<EcoString, ModuleDef> {
        self.module_defs.drain().collect()
    }

    /// Imports a module under a given alias.
    ///
    /// # Parameters
    /// - `address: &Address` — The source location for error reporting.
    /// - `name: EcoString` — The alias under which the module should be imported.
    /// - `module: RcPtr<Module>` — The module to import.
    ///
    /// # Errors
    /// - Raises `TypeckError::ModuleIsAlreadyImportedAs` if the alias is already used.
    ///
    #[instrument(skip(address), level = "trace")]
    pub fn import_as(&mut self, address: &Address, name: EcoString, module: RcPtr<Module>) {
        match self.imported_modules.get(&name) {
            Some(module) => bail!(TypeckError::ModuleIsAlreadyImportedAs {
                src: address.source.clone(),
                span: address.span.clone().into(),
                m: module.name.clone(),
                name: name.clone()
            }),
            None => self.imported_modules.insert(name, module),
        };
    }

    /// Imports specific names (definitions) from a module.
    ///
    /// # Parameters
    /// * `icx: &mut InferCx` — Inference context used for pretty printing definitions.
    /// * `address: &Address` — Source location for error reporting.
    /// * `names: Vec<EcoString>` — Names of definitions to import from the module.
    /// * `module: RcPtr<Module>` — The module to import from.
    ///
    /// # Behavior
    /// For each name:
    /// 1. Checks if the name exists in the module's fields.
    /// 2. Checks if the name was already imported from another module.
    /// 3. Inserts the definition into `imported_defs` if both checks pass.
    ///
    /// # Errors
    /// - `TypeckError::ModuleFieldIsNotDefined` if the name does not exist in the module.
    /// - `TypeckError::DefIsAlreadyImported` if the name has already been imported.
    ///
    #[instrument(skip(icx, address), level = "trace")]
    pub fn import_for(
        &mut self,
        icx: &mut InferCx,
        address: &Address,
        names: Vec<EcoString>,
        module: RcPtr<Module>,
    ) {
        for name in names {
            match module.fields.get(&name) {
                Some(def) => match self.imported_defs.get(&name) {
                    Some(already) => bail!(TypeckError::DefIsAlreadyImported {
                        src: address.source.clone(),
                        span: address.span.clone().into(),
                        name: name.clone(),
                        def: already.pretty(icx),
                    }),
                    None => {
                        self.imported_defs.insert(name, def.clone());
                    }
                },
                None => {
                    bail!(TypeckError::ModuleFieldIsNotDefined {
                        src: address.source.clone(),
                        span: address.span.clone().into(),
                        m: module.name.clone(),
                        field: name
                    })
                }
            }
        }
    }
}
