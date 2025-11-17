/// Imports
use crate::{
    cx::module::ModuleCx,
    errors::TypeckError,
    inference::equation::Equation,
    typ::{
        def::{ModuleDef, TypeDef},
        res::Res,
        typ::{Enum, EnumVariant, Field, Function, Parameter, Struct, Typ, WithPublicity},
    },
};
use ecow::EcoString;
use indexmap::IndexMap;
use std::rc::Rc;
use watt_ast::ast::{
    self, Block, Declaration, Dependency, Either, EnumConstructor, Expression, Publicity, TypePath,
    UseKind,
};
use watt_common::{address::Address, bail};

/// Declaraton analyze
impl<'pkg, 'cx> ModuleCx<'pkg, 'cx> {
    /// Analyzes struct fields
    fn late_analyze_struct(&mut self, location: Address, name: EcoString, fields: Vec<ast::Field>) {
        // Requesting struct
        let ty = match self.resolver.resolve_type(&location, &name) {
            TypeDef::Struct(ty) => ty,
            _ => unreachable!(),
        };
        let borrowed = ty.borrow();

        // Repushing generics
        self.solver
            .hydrator
            .generics
            .re_push_scope(borrowed.generics.clone());

        // Inferencing fields
        let new_struct = Struct {
            location: borrowed.location.clone(),
            uid: borrowed.uid,
            name: borrowed.name.clone(),
            generics: borrowed.generics.clone(),
            fields: fields
                .into_iter()
                .map(|f| Field {
                    name: f.name,
                    location: f.location,
                    typ: self.infer_type_annotation(f.typ),
                })
                .collect(),
        };
        drop(borrowed);
        *ty.borrow_mut() = new_struct;

        // Popping generics
        self.solver.hydrator.generics.pop_scope();
    }

    /// Analyzes enum variants
    fn late_analyze_enum(
        &mut self,
        location: Address,
        name: EcoString,
        variants: Vec<EnumConstructor>,
    ) {
        // Requesting enum
        let en = match self.resolver.resolve_type(&location, &name) {
            TypeDef::Enum(en) => en,
            _ => unreachable!(),
        };
        let borrowed = en.borrow();

        // Repushing generics
        self.solver
            .hydrator
            .generics
            .re_push_scope(borrowed.generics.clone());

        // Inferencing fields
        let new_enum = Enum {
            location: borrowed.location.clone(),
            uid: borrowed.uid,
            name: borrowed.name.clone(),
            generics: borrowed.generics.clone(),
            variants: variants
                .into_iter()
                .map(|v| EnumVariant {
                    location: v.location,
                    name: v.name,
                    fields: v
                        .params
                        .into_iter()
                        .map(|p: ast::Parameter| Field {
                            location: p.location,
                            name: p.name,
                            typ: self.infer_type_annotation(p.typ),
                        })
                        .collect(),
                })
                .collect(),
        };
        drop(borrowed);
        *en.borrow_mut() = new_enum;

        // Popping generics
        self.solver.hydrator.generics.pop_scope();
    }

    /// Analyzes funciton body.
    fn late_analyze_function_decl(
        &mut self,
        location: Address,
        publicity: Publicity,
        name: EcoString,
        params: Vec<ast::Parameter>,
        body: Either<Block, Expression>,
        ret_type: Option<TypePath>,
    ) {
        // Requesting function
        let f = match self.resolver.resolve(&location, &name) {
            Res::Value(Typ::Function(f)) => f,
            _ => unreachable!(),
        };

        // Pushing generics
        self.solver
            .hydrator
            .generics
            .re_push_scope(f.generics.clone());

        // inferring return type
        let ret = ret_type.map_or(Typ::Unit, |t| self.infer_type_annotation(t));

        // inferred params
        let params = params
            .into_iter()
            .map(|p| {
                (
                    p.name,
                    Parameter {
                        location: p.location,
                        typ: self.infer_type_annotation(p.typ),
                    },
                )
            })
            .collect::<IndexMap<EcoString, Parameter>>();

        // creating and defining function
        let function = Function {
            location: location.clone(),
            name: name.clone(),
            generics: f.generics.clone(),
            params: params.clone().into_values().collect(),
            ret: ret.clone(),
        };
        self.resolver.define_module(
            &location,
            &name,
            ModuleDef::Function(WithPublicity {
                publicity,
                value: Rc::new(function),
            }),
            true,
        );

        // pushing new scope
        self.resolver.push_rib();

        // defining params in new scope
        params.iter().for_each(|p| {
            self.resolver
                .define_local(&location, p.0, p.1.typ.clone(), false)
        });

        // inferring body
        let (block_location, inferred_block) = match body {
            Either::Left(block) => (block.location.clone(), self.infer_block(block)),
            Either::Right(expr) => (expr.location(), self.infer_expr(expr)),
        };
        self.solver.solve(Equation::Unify(
            (location, ret),
            (block_location, inferred_block),
        ));
        self.resolver.pop_rib();

        // Popping generics
        self.solver.hydrator.generics.pop_scope();
    }

    /// Analyzes extern funciton body.
    fn late_analyze_extern_decl(
        &mut self,
        location: Address,
        publicity: Publicity,
        name: EcoString,
        params: Vec<ast::Parameter>,
        ret_type: Option<TypePath>,
    ) {
        // Requesting function
        let f = match self.resolver.resolve(&location, &name) {
            Res::Value(Typ::Function(f)) => f,
            _ => unreachable!(),
        };

        // Pushing generics
        self.solver
            .hydrator
            .generics
            .re_push_scope(f.generics.clone());

        // inferring return type
        let ret = ret_type.map_or(Typ::Unit, |t| self.infer_type_annotation(t));

        // inferred params
        let params = params
            .into_iter()
            .map(|p| {
                (
                    p.name,
                    Parameter {
                        location: p.location,
                        typ: self.infer_type_annotation(p.typ),
                    },
                )
            })
            .collect::<IndexMap<EcoString, Parameter>>();

        // creating and defining function
        let function = Function {
            location: location.clone(),
            name: name.clone(),
            generics: f.generics.clone(),
            params: params.clone().into_values().collect(),
            ret: ret.clone(),
        };
        self.resolver.define_module(
            &location,
            &name,
            ModuleDef::Function(WithPublicity {
                publicity,
                value: Rc::new(function),
            }),
            true,
        );

        // Popping generics
        self.solver.hydrator.generics.pop_scope();
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
    ///
    fn late_define_const(
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

    /// Late declaration analysis
    pub fn late_analyze_declaration(&mut self, declaration: Declaration) {
        match declaration {
            Declaration::TypeDeclaration {
                location,
                name,
                fields,
                ..
            } => self.late_analyze_struct(location, name, fields),
            Declaration::EnumDeclaration {
                location,
                name,
                variants,
                ..
            } => self.late_analyze_enum(location, name, variants),
            Declaration::Function {
                location,
                publicity,
                name,
                params,
                body,
                typ,
                ..
            } => self.late_analyze_function_decl(location, publicity, name, params, body, typ),
            Declaration::ExternFunction {
                location,
                publicity,
                name,
                params,
                typ,
                ..
            } => self.late_analyze_extern_decl(location, publicity, name, params, typ),
            Declaration::Const {
                location,
                publicity,
                name,
                value,
                typ
            } => self.late_define_const(location, publicity, name, value, typ),
        }
    }

    /// Performs import
    pub fn perform_import(&mut self, import: Dependency) {
        match self.package.root.modules.get(&import.path.module) {
            Some(module) => match import.kind {
                UseKind::AsName(name) => {
                    self.resolver
                        .import_as(&import.location, name, module.clone())
                }
                UseKind::ForNames(names) => {
                    self.resolver
                        .import_for(&import.location, names, module.clone())
                }
            },
            None => bail!(TypeckError::ImportOfUnknownModule {
                src: self.module.source.clone(),
                span: import.location.span.into(),
                m: import.path.module
            }),
        };
    }
}
