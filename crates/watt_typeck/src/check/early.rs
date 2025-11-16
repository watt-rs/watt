use std::{cell::RefCell, rc::Rc};

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
use watt_ast::ast::{Declaration, Expression, Publicity, TypePath};
use watt_common::address::Address;

/// Early analyze process
impl<'pkg, 'cx> ModuleCx<'pkg, 'cx> {
    /// Early defines struct by name
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
        // Popping generics
        self.solver.hydrator.generics.pop_scope();
    }

    /// Early defines function just by name
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
        // Popping generics
        self.solver.hydrator.generics.pop_scope();
    }

    /// Early defines enum just by name
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
        // Popping generics
        self.solver.hydrator.generics.pop_scope();
    }

    /// Early defines extern fn just by name
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
        // Popping generics
        self.solver.hydrator.generics.pop_scope();
    }

    /// Defines const variable
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

    /// Early defines declaration
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
                generics,
                name,
                publicity,
                ..
            } => self.early_define_extern(location, publicity, generics, name),
            Declaration::Function {
                location,
                publicity,
                generics,
                name,
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
