use crate::cx::module::ModuleCx;
use crate::typ::def::ModuleDef;
use crate::typ::typ::{Function, Parameter, Typ, WithPublicity};
use ecow::EcoString;
/// Imports
use std::rc::Rc;
use watt_ast::ast;
use watt_ast::ast::{FnDeclaration, Publicity, TypePath};
use watt_common::address::Address;

/// Performs the “early” pass of module analysis.
///
/// The early phase registers symbols (types, enums, functions, externals)
/// in the module scope *by signature and generics only*, without inspecting their internals.
///
/// This ensures that forward references are allowed:
/// all types and functions with inferred signatures
/// become visible before later semantic analysis begins.
///
/// No fields, parameters or bodies are analyzed here.
/// Only namespace entry creation happens.
///
impl<'pkg, 'cx> ModuleCx<'pkg, 'cx> {
    /// Registers a function or extern function
    /// symbol in the module before its body is analyzed.
    ///
    /// Everything except function body will be analyzed.
    /// [`late_analyze_function_decl`] performs full semantic analysis.
    ///
    pub(crate) fn early_define_fn(
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

    /// Dispatches early-phase analysis for any kind of function declaration.
    ///
    /// Each fn declaration type is handled by the corresponding `early_define_*`
    /// method. This ensures all top-level functions are registered and
    /// its params and return type are inferred before any
    /// “late” semantic analysis runs.
    ///
    pub(crate) fn early_analyze_fn_decl(&mut self, declaration: &FnDeclaration) {
        match declaration.clone() {
            FnDeclaration::ExternFunction {
                location,
                name,
                publicity,
                generics,
                params,
                typ,
                ..
            }
            | FnDeclaration::Function {
                location,
                publicity,
                name,
                generics,
                params,
                typ,
                ..
            } => self.early_define_fn(location, publicity, generics, params, typ, name),
        }
    }
}
