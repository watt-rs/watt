/// Imports
use crate::{
    cx::module::ModuleCx,
    errors::{TypeckError, TypeckRelated},
    typ::{
        def::{ModuleDef, TypeDef},
        typ::{Enum, Function, GenericArgs, Parameter, PreludeType, Struct, Typ},
    },
};
use ecow::EcoString;
use std::{cell::RefCell, rc::Rc};
use watt_ast::ast::{Publicity, TypePath};
use watt_common::{address::Address, bail};

/// Implementation
impl<'pkg, 'cx> ModuleCx<'pkg, 'cx> {
    /// Ensures no generic parameters given
    fn ensure_no_generics<F>(&self, location: &Address, got: usize, typ: F) -> Typ
    where
        F: FnOnce() -> Typ,
    {
        if got == 0 {
            typ()
        } else {
            bail!(TypeckError::ArityMissmatch {
                related: vec![TypeckRelated::Here {
                    src: location.source.clone(),
                    span: location.span.clone().into()
                }],
                expected: 0,
                got
            })
        }
    }

    /// Checks generic parameters arity
    fn check_generic_params_arity(&self, location: &Address, expected: usize, got: usize) {
        if expected != got {
            bail!(TypeckError::ArityMissmatch {
                related: vec![TypeckRelated::Here {
                    src: location.source.clone(),
                    span: location.span.clone().into()
                }],
                expected,
                got
            })
        }
    }

    /// Infers a local type (built-in or user-defined).
    fn infer_local_type_path(
        &mut self,
        location: Address,
        name: EcoString,
        generics: Vec<TypePath>,
    ) -> Typ {
        match name.as_str() {
            // Prelude types
            "int" => self
                .ensure_no_generics(&location, generics.len(), || Typ::Prelude(PreludeType::Int)),
            "float" => self.ensure_no_generics(&location, generics.len(), || {
                Typ::Prelude(PreludeType::Float)
            }),
            "bool" => self.ensure_no_generics(&location, generics.len(), || {
                Typ::Prelude(PreludeType::Bool)
            }),
            "string" => self.ensure_no_generics(&location, generics.len(), || {
                Typ::Prelude(PreludeType::String)
            }),
            "unit" => self.ensure_no_generics(&location, generics.len(), || Typ::Unit),

            // User-defined types
            _ => match self.solver.hydrator.generics.get(&name) {
                Some(id) => Typ::Generic(id),
                None => match self.resolver.resolve_type(&location, &name) {
                    TypeDef::Enum(en) => self.instantiate_enum_type(&location, en, generics),
                    TypeDef::Struct(st) => self.instantiate_struct_type(&location, st, generics),
                },
            },
        }
    }

    /// Infers a type imported from another module.
    ///
    /// Performs visibility checks and handles both enums and structs.
    fn infer_module_type_path(
        &mut self,
        location: Address,
        module: EcoString,
        name: EcoString,
        generics: Vec<TypePath>,
    ) -> Typ {
        let m = self.resolver.resolve_module(&module);

        let def = match m.fields.get(&name) {
            Some(ModuleDef::Type(def)) if def.publicity != Publicity::Private => def.clone(),
            Some(ModuleDef::Type(def)) => bail!(TypeckError::TypeIsPrivate {
                src: self.module.source.clone(),
                span: location.span.into(),
                def: def.value.clone()
            }),
            Some(ModuleDef::Const(_)) | Some(ModuleDef::Function(_)) => {
                bail!(TypeckError::CouldNotUseValueAsType {
                    src: self.module.source.clone(),
                    span: location.span.into(),
                    v: name
                })
            }
            None => bail!(TypeckError::TypeIsNotDefined {
                src: self.module.source.clone(),
                span: location.span.into(),
                t: format!("{module}.{name}").into()
            }),
        };

        match &def.value {
            TypeDef::Enum(en) => self.instantiate_enum_type(&location, en.clone(), generics),
            TypeDef::Struct(st) => self.instantiate_struct_type(&location, st.clone(), generics),
        }
    }

    /// Infers a function type annotation like `fn(int, string): bool`.
    fn infer_function_type_path(
        &mut self,
        location: Address,
        params: Vec<TypePath>,
        ret: Option<Box<TypePath>>,
    ) -> Typ {
        Typ::Function(Rc::new(Function {
            location,
            name: EcoString::from("$annotated"),
            generics: Vec::new(),
            params: params
                .into_iter()
                .map(|p| Parameter {
                    location: p.location(),
                    typ: self.infer_type_annotation(p),
                })
                .collect(),
            ret: ret.map_or(Typ::Unit, |t| self.infer_type_annotation(*t)),
        }))
    }

    /// Instantiates an enum type with its generic parameters.
    fn instantiate_enum_type(
        &mut self,
        location: &Address,
        en: Rc<RefCell<Enum>>,
        generics: Vec<TypePath>,
    ) -> Typ {
        self.check_generic_params_arity(location, en.borrow().generics.len(), generics.len());

        let substitutions = GenericArgs {
            subtitutions: en
                .borrow()
                .generics
                .iter()
                .zip(generics)
                .map(|(param, arg)| {
                    let ty = self.infer_type_annotation(arg);
                    (param.id, ty)
                })
                .collect(),
        };

        Typ::Enum(en, substitutions)
    }

    /// Instantiates a struct type with its generic parameters.
    fn instantiate_struct_type(
        &mut self,
        location: &Address,
        st: Rc<RefCell<Struct>>,
        generics: Vec<TypePath>,
    ) -> Typ {
        self.check_generic_params_arity(location, st.borrow().generics.len(), generics.len());

        let substitutions = GenericArgs {
            subtitutions: st
                .borrow()
                .generics
                .iter()
                .zip(generics)
                .map(|(param, arg)| {
                    let ty = self.infer_type_annotation(arg);
                    (param.id, ty)
                })
                .collect(),
        };

        Typ::Struct(st, substitutions)
    }

    /// Infers a type annotation from a [`TypePath`].
    ///
    /// ## This function handles:
    /// - Prelude (built-in) types: `int`, `float`, `bool`, `string`, `()`
    /// - User-defined types (enums and structs)
    /// - Module-qualified types (e.g. `math.Vector`)
    /// - Function type expressions (e.g. `(int, float) -> bool`)
    /// - Unit type
    ///
    /// Each branch validates generic parameters count and ensures
    /// access visibility for types imported from other modules.
    /// 
    pub(crate) fn infer_type_annotation(&mut self, path: TypePath) -> Typ {
        match path.clone() {
            TypePath::Local {
                location,
                name,
                generics,
            } => self.infer_local_type_path(location, name, generics),
            TypePath::Module {
                location,
                module,
                name,
                generics,
            } => self.infer_module_type_path(location, module, name, generics),
            TypePath::Function {
                location,
                params,
                ret,
            } => self.infer_function_type_path(location, params, ret),
            TypePath::Unit { .. } => Typ::Unit,
        }
    }
}
