/// Imports
use crate::{
    cx::module::ModuleCx,
    inference::equation::Equation,
    typ::{
        def::{ModuleDef, TypeDef},
        typ::{Enum, Function, Struct, Typ, WithPublicity},
    },
};
use ecow::EcoString;
use std::{cell::RefCell, rc::Rc};
use watt_ast::ast::{Declaration, Expression, Publicity, TypePath};
use watt_common::address::Address;

/// Performs the “early” pass of module analysis.
///
/// The early phase registers symbols (types, enums, functions, externs, consts)
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
    /// but without performing any semantic checks.
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
    /// Only the function’s name, location, generics and publicity are stored.
    /// Parameters and return type remain empty until
    /// [`late_analyze_function_decl`] performs full semantic analysis.
    ///
    fn early_define_function_decl(
        &mut self,
        location: Address,
        publicity: Publicity,
        generics: Vec<EcoString>,
        name: EcoString,
    ) {
        // Pushing generics
        let generics = self.solver.hydrator.generics.push_scope(generics);
        // Generating function
        let function = Rc::new(Function {
            location: location.clone(),
            name: name.clone(),
            generics,
            params: Vec::new(),
            ret: Typ::Unit,
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
    /// this creates a bare [`Function`] with no parameters and `Unit` return type.
    /// Further semantic passes do not analyze extern functions.
    ///
    fn early_define_extern(
        &mut self,
        location: Address,
        publicity: Publicity,
        generics: Vec<EcoString>,
        name: EcoString,
    ) {
        // Pushing generics
        let generics = self.solver.hydrator.generics.push_scope(generics);
        // Generating function
        let function = Rc::new(Function {
            location: location.clone(),
            name: name.clone(),
            generics,
            params: Vec::new(),
            ret: Typ::Unit,
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

    /// Defines a constant value.
    ///
    /// Steps:
    /// 1. Infer the annotated type (`typ`).
    /// 2. Infer the expression value.
    /// 3. Emit a unification equation requiring that the inferred value type
    ///    equals the annotated type.
    /// 4. Register the constant under its name.
    ///
    /// Constants do not introduce generics or additional scopes.
    /// Further semantic passes do not analyze constants.
    ///
    fn define_const(
        &mut self,
        location: Address,
        publicity: Publicity,
        name: EcoString,
        value: Expression,
        typ: TypePath,
    ) {
        // Const inference
        let annotated_location = typ.location();
        let annotated = self.infer_type_annotation(typ);
        let inferred_location = value.location();
        let inferred = self.infer_expr(value);
        self.solver.solve(Equation::Unify(
            (annotated_location, annotated.clone()),
            (inferred_location, inferred),
        ));

        // Defining constant
        self.resolver.define_module(
            &location,
            &name,
            ModuleDef::Const(WithPublicity {
                publicity,
                value: annotated,
            }),
            false,
        );
    }

    /// Dispatches early-phase definition for any kind of declaration.
    ///
    /// Each declaration type is handled by the corresponding `early_define_*`
    /// method. This ensures all top-level symbols are registered before any
    /// “late” semantic analysis runs.
    ///
    pub(crate) fn early_define(&mut self, declaration: &Declaration) {
        // Matching declaration
        match declaration.clone() {
            Declaration::TypeDeclaration {
                location,
                name,
                publicity,
                generics,
                ..
            } => self.early_define_struct(location, publicity, generics, name),
            Declaration::EnumDeclaration {
                location,
                name,
                publicity,
                generics,
                ..
            } => self.early_define_enum(location, publicity, generics, name),
            Declaration::ExternFunction {
                location,
                name,
                publicity,
                generics,
                ..
            } => self.early_define_extern(location, publicity, generics, name),
            Declaration::Function {
                location,
                publicity,
                name,
                generics,
                ..
            } => self.early_define_function_decl(location, publicity, generics, name),
            Declaration::Const {
                location,
                publicity,
                name,
                value,
                typ,
            } => self.define_const(location, publicity, name, value, typ),
        }
    }
}
