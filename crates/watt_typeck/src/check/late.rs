/// Imports
use crate::{
    cx::module::ModuleCx,
    errors::TypeckError,
    resolve::{
        res::Res,
        resolve::{Def, ModDef},
        rib::RibKind,
    },
    typ::{CustomType, Function, Parameter, Struct, Typ, WithPublicity},
    unify::Equation,
};
use ecow::EcoString;
use indexmap::IndexMap;
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use watt_ast::ast::{
    self, Block, Declaration, Dependency, Either, Expression, Field, Method, Publicity, TypePath,
    UseKind,
};
use watt_common::{address::Address, bail};

/// Declaraton analyze
impl<'pkg, 'cx> ModuleCx<'pkg, 'cx> {
    /// Analyzes method
    #[allow(clippy::too_many_arguments)]
    fn analyze_method(
        &mut self,
        location: Address,
        name: EcoString,
        strct: Rc<RefCell<Struct>>,
        publicity: Publicity,
        params: Vec<ast::Parameter>,
        body: Either<Block, Expression>,
        ret_type: Option<TypePath>,
    ) {
        // inferring return type
        let ret = ret_type.map_or(Typ::Unit, |t| self.infer_type_annotation(t));
        self.resolver.push_rib(RibKind::Function);

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

        // Defining params
        params.iter().for_each(|p| {
            self.resolver
                .define(&location, &p.0, Def::Local(p.1.typ.clone()), false)
        });

        // creating and defining function
        let function = Function {
            source: self.module.source.clone(),
            location: location.clone(),
            uid: self.fresh_id(),
            name: name.clone(),
            params: params.into_values().collect(),
            ret: ret.clone(),
        };

        // defining function, if not already defined
        if strct.borrow().env.contains_key(&name) {
            bail!(TypeckError::MethodIsAlreadyDefined {
                src: self.module.source.clone(),
                span: location.span.into(),
                m: name
            })
        } else {
            strct.borrow_mut().env.insert(
                name,
                WithPublicity {
                    publicity,
                    value: Typ::Function(Rc::new(function)),
                },
            );
        }

        self.resolver.define(
            &location,
            &"self".into(),
            Def::Local(Typ::Struct(strct.clone())),
            false,
        );

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
    }

    /// Reanalyzes struct and analyzes it's methods and fields
    fn late_analyze_struct(
        &mut self,
        location: Address,
        name: EcoString,
        publicity: Publicity,
        params: Vec<ast::Parameter>,
        fields: Vec<Field>,
        methods: Vec<Method>,
    ) {
        // Requesting struct
        let typ = match self.resolver.resolve(&location, &name) {
            Res::Custom(ty) => match ty {
                CustomType::Struct(ty) => ty,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        };

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

        // construction type
        let type_ = Rc::new(RefCell::new(Struct {
            source: location.source.clone(),
            location: location.clone(),
            uid: typ.borrow().uid,
            name: name.clone(),
            params: params.clone().into_values().collect(),
            env: HashMap::new(),
        }));

        // defining type, if not already defined
        self.resolver.define(
            &location,
            &name,
            Def::Module(ModDef::CustomType(WithPublicity {
                publicity,
                value: CustomType::Struct(type_.clone()),
            })),
            true,
        );

        // params env start
        self.resolver.push_rib(RibKind::ConstructorParams);

        // params
        params.into_iter().for_each(|p| {
            self.resolver
                .define(&p.1.location, &p.0, Def::Local(p.1.typ), false);
        });

        // fields env start
        self.resolver.push_rib(RibKind::Fields);

        // analyzing fields
        fields.iter().cloned().for_each(|f| {
            let inferred_location = f.value.location();
            let inferred = self.infer_expr(f.value);
            let type_annotation_location = f.typ.location();
            let type_annotation = self.infer_type_annotation(f.typ);
            self.solver.solve(Equation::Unify(
                (inferred_location, inferred),
                (type_annotation_location, type_annotation.clone()),
            ));
            self.resolver
                .define(&location, &f.name, Def::Local(type_annotation), false)
        });

        // fields env end
        let analyzed_fields = match self.resolver.pop_rib() {
            Some(fields) => fields.1,
            None => bail!(TypeckError::EnvironmentsStackIsEmpty),
        };

        // params env end
        self.resolver.pop_rib();

        // adding fields to type env
        let mut borrowed = type_.borrow_mut();
        fields.iter().cloned().for_each(|f| {
            borrowed.env.insert(
                f.name.clone(),
                WithPublicity {
                    publicity: f.publicity,
                    value: analyzed_fields.get(&f.name).unwrap().clone(),
                },
            );
        });
        drop(borrowed);

        // type env start
        self.resolver.push_rib(RibKind::Struct(type_.clone()));

        // adding methods to type env
        methods.into_iter().for_each(|m| {
            self.analyze_method(
                m.location,
                m.name,
                type_.clone(),
                m.publicity,
                m.params,
                m.body,
                m.typ,
            );
        });

        // type env end
        self.resolver.pop_rib();
    }

    /// Reanalyzes funciton and analyzes it's body.
    fn late_analyze_function_decl(
        &mut self,
        location: Address,
        publicity: Publicity,
        name: EcoString,
        params: Vec<ast::Parameter>,
        body: Either<Block, Expression>,
        ret_type: Option<TypePath>,
    ) {
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
            source: self.module.source.clone(),
            location: location.clone(),
            uid: self.fresh_id(),
            name: name.clone(),
            params: params.clone().into_values().collect(),
            ret: ret.clone(),
        };
        self.resolver.define(
            &location,
            &name,
            Def::Module(ModDef::Variable(WithPublicity {
                publicity,
                value: Typ::Function(Rc::new(function)),
            })),
            true,
        );

        // pushing new scope
        self.resolver.push_rib(RibKind::Function);

        // defining params in new scope
        params.iter().for_each(|p| {
            self.resolver
                .define(&location, p.0, Def::Local(p.1.typ.clone()), false)
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
    }

    /// Analyzes define
    pub(crate) fn late_analyze_define(
        &mut self,
        location: Address,
        publicity: Publicity,
        name: EcoString,
        value: Expression,
        typ: Option<TypePath>,
    ) {
        let inferred_value = self.infer_expr(value);
        match typ {
            Some(annotated_path) => {
                let annotated_location = annotated_path.location();
                let annotated = self.infer_type_annotation(annotated_path);
                self.solver.solve(Equation::Unify(
                    (annotated_location, annotated.clone()),
                    (location.clone(), inferred_value.clone()),
                ));
                self.resolver.define(
                    &location,
                    &name,
                    Def::Module(ModDef::Variable(WithPublicity {
                        publicity,
                        value: annotated,
                    })),
                    false,
                )
            }
            None => self.resolver.define(
                &location,
                &name,
                Def::Module(ModDef::Variable(WithPublicity {
                    publicity,
                    value: inferred_value,
                })),
                false,
            ),
        }
    }

    /// Late declaration analysis
    pub fn late_analyze_declaration(&mut self, declaration: Declaration) {
        match declaration {
            Declaration::TypeDeclaration {
                location,
                name,
                publicity,
                constructor,
                fields,
                methods,
            } => self.late_analyze_struct(location, name, publicity, constructor, fields, methods),
            Declaration::VarDef {
                location,
                publicity,
                name,
                value,
                typ,
            } => self.late_analyze_define(location, publicity, name, value, typ),
            Declaration::Function {
                location,
                publicity,
                name,
                params,
                body,
                typ,
            } => self.late_analyze_function_decl(location, publicity, name, params, body, typ),
            // Extern functions, enums and traits does not need any late analysys
            _ => {}
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
