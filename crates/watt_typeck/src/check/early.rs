use crate::typ::typ::Parameter;
/// Imports
use crate::{
    cx::module::ModuleCx,
    typ::{
        def::{ModuleDef, TypeDef},
        typ::{Enum, Function, Struct, Typ, WithPublicity},
    },
};
use ecow::EcoString;
use std::{cell::RefCell, rc::Rc};
use watt_ast::ast;
use watt_ast::ast::{FnDeclaration, Publicity, TypeDeclaration, TypePath};
use watt_common::address::Address;

/// Performs the “early” pass of module analysis.
///
/// The early phase registers symbols (types, enums, functions, externs)
/// in the module scope *by name and generics only*, without inspecting their internals.
///
/// This ensures that forward references are allowed:
/// all types and functions become visible before later semantic analysis begins.
///
/// No fields, parameters or bodies are analyzed here.
/// Only namespace entry creation happens.
impl<'pkg, 'cx> ModuleCx<'pkg, 'cx> {
    /// Registers a struct name in the module before its fields are analyzed.
    ///
    /// This creates a placeholder [`Struct`] with:
    /// - empty fields,
    /// - generic params,
    /// - fresh `uid`,
    ///   but without performing any semantic checks.
    ///
    /// The full struct body will later be populated in
    /// [`late_analyze_struct`].
    ///
    fn early_define_struct(
        &mut self,
        location: Address,
        publicity: Publicity,
        generics: Vec<EcoString>,
        name: EcoString,
    ) {
        // Pushing generics
        let generics = self.solver.hydrator.generics.push_scope(generics);
        // Generating struct
        let strct = TypeDef::Struct(Rc::new(RefCell::new(Struct {
            location: location.clone(),
            uid: self.fresh_id(),
            name: name.clone(),
            generics,
            fields: Vec::new(),
        })));
        // Popping generics
        self.solver.hydrator.generics.pop_scope();
        // Defining struct
        self.resolver.define_module(
            &location,
            &name,
            ModuleDef::Type(WithPublicity {
                publicity,
                value: strct,
            }),
            false,
        );
    }

    /// Registers a function symbol in the module before its body is analyzed.
    ///
    /// Everything except function body will be analyzed.
    /// [`late_analyze_function_decl`] performs full semantic analysis.
    ///
    fn early_define_function_decl(
        &mut self,
        location: Address,
        publicity: Publicity,
        generics: Vec<EcoString>,
        params: Vec<ast::Parameter>,
        typ: Option<TypePath>,
        name: EcoString,
    ) {
        // Pushing generics
        let generics = self.solver.hydrator.generics.push_scope(generics);
        // Generating function
        let function = Rc::new(Function {
            location: location.clone(),
            name: name.clone(),
            generics,
            params: params
                .into_iter()
                .map(|p| Parameter {
                    location: p.location.clone(),
                    typ: self.infer_type_annotation(p.typ),
                })
                .collect(),
            ret: typ.map_or(Typ::Unit, |it| self.infer_type_annotation(it)),
        });
        // Popping generics
        self.solver.hydrator.generics.pop_scope();
        // Defining function
        self.resolver.define_module(
            &location,
            &name,
            ModuleDef::Function(WithPublicity {
                publicity,
                value: function,
            }),
            false,
        );
    }

    /// Registers an enum name in the module before its variants are analyzed.
    ///
    /// The enum is inserted as a placeholder containing:
    /// - generics params,
    /// - no variants,
    /// - fresh `uid`.
    ///
    /// Variants and their fields are added later during
    /// [`late_analyze_enum`].
    ///
    fn early_define_enum(
        &mut self,
        location: Address,
        publicity: Publicity,
        generics: Vec<EcoString>,
        name: EcoString,
    ) {
        // Pushing generics
        let generics = self.solver.hydrator.generics.push_scope(generics);
        // Generating enum
        let en = TypeDef::Enum(Rc::new(RefCell::new(Enum {
            location: location.clone(),
            uid: self.fresh_id(),
            name: name.clone(),
            generics,
            variants: Vec::new(),
        })));
        // Popping generics
        self.solver.hydrator.generics.pop_scope();
        // Defining enum
        self.resolver.define_module(
            &location,
            &name,
            ModuleDef::Type(WithPublicity {
                publicity,
                value: en,
            }),
            false,
        );
    }

    /// Registers an `extern` function.
    ///
    /// Since extern functions have no bodies, their analysis is trivial:
    /// Function generics, params, return type be analyzed.
    /// [`late_analyze_function_decl`] performs full semantic analysis.
    /// Further semantic passes do not analyze extern functions.
    ///
    fn early_define_extern(
        &mut self,
        location: Address,
        publicity: Publicity,
        generics: Vec<EcoString>,
        params: Vec<ast::Parameter>,
        typ: Option<TypePath>,
        name: EcoString,
    ) {
        // Pushing generics
        let generics = self.solver.hydrator.generics.push_scope(generics);
        // Generating function
        let function = Rc::new(Function {
            location: location.clone(),
            name: name.clone(),
            generics,
            params: params
                .into_iter()
                .map(|p| Parameter {
                    location: p.location.clone(),
                    typ: self.infer_type_annotation(p.typ),
                })
                .collect(),
            ret: typ.map_or(Typ::Unit, |it| self.infer_type_annotation(it)),
        });
        // Popping generics
        self.solver.hydrator.generics.pop_scope();
        // defining function
        self.resolver.define_module(
            &location,
            &name,
            ModuleDef::Function(WithPublicity {
                publicity,
                value: function,
            }),
            false,
        );
    }

    /// Dispatches early-phase definition for any kind of type declaration.
    ///
    /// Each declaration type is handled by the corresponding `early_define_*`
    /// method. This ensures all top-level symbols are registered before any
    /// “late” semantic analysis runs.
    ///
    pub(crate) fn early_define_type(&mut self, declaration: &TypeDeclaration) {
        // Matching declaration
        match declaration.clone() {
            TypeDeclaration::Struct {
                location,
                name,
                publicity,
                generics,
                ..
            } => self.early_define_struct(location, publicity, generics, name),
            TypeDeclaration::Enum {
                location,
                name,
                publicity,
                generics,
                ..
            } => self.early_define_enum(location, publicity, generics, name),
        }
    }

    /// Dispatches early-phase definition for any kind of function declaration.
    ///
    /// Each declaration type is handled by the corresponding `early_define_*`
    /// method. This ensures all top-level functions are registered and
    /// its params and return type are inferred before any
    /// “late” semantic analysis runs.
    ///
    pub(crate) fn early_define_fn(&mut self, declaration: &FnDeclaration) {
        match declaration.clone() {
            FnDeclaration::ExternFunction {
                location,
                name,
                publicity,
                generics,
                params,
                typ,
                ..
            } => self.early_define_extern(location, publicity, generics, params, typ, name),
            FnDeclaration::Function {
                location,
                publicity,
                name,
                generics,
                params,
                typ,
                ..
            } => self.early_define_function_decl(location, publicity, generics, params, typ, name),
        }
    }
}
