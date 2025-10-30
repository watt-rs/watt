/// Imports
use crate::{
    cx::module::ModuleCx,
    errors::TypeckError,
    resolve::{
        resolve::{Def, ModDef},
        rib::RibKind,
    },
    typ::{CustomType, Enum, EnumVariant, Function, Trait, Typ, Type, WithPublicity},
    unify::Equation,
};
use ecow::EcoString;
use std::{cell::RefCell, collections::HashMap};
use watt_ast::ast::{
    self, Block, Declaration, Dependency, Either, EnumConstructor, Expression, Parameter,
    Publicity, TypePath, UseKind,
};
use watt_common::{address::Address, bail, rc_ptr::RcPtr};

/// Declaraton analyze
impl<'pkg, 'cx> ModuleCx<'pkg, 'cx> {
    /// Analyzes method
    #[allow(clippy::too_many_arguments)]
    fn analyze_method(
        &mut self,
        location: Address,
        name: EcoString,
        type_: RcPtr<RefCell<Type>>,
        publicity: Publicity,
        params: Vec<Parameter>,
        body: Either<Block, Expression>,
        ret_type: Option<TypePath>,
    ) {
        // inferring return type
        let ret = ret_type.map_or(Typ::Unit, |t| self.infer_type_annotation(t));
        self.resolver.push_rib(RibKind::Function);

        // inferring params
        let params = params
            .into_iter()
            .map(|p| (p.name, self.infer_type_annotation(p.typ.clone())))
            .collect::<HashMap<EcoString, Typ>>();

        params.iter().for_each(|p| {
            self.resolver
                .define(&location, p.0, Def::Local(p.1.clone()))
        });

        // creating and defining function
        let function = Function {
            source: self.module.source.clone(),
            location: location.clone(),
            name: name.clone(),
            params: params.into_values().collect::<Vec<Typ>>(),
            ret: ret.clone(),
        };

        // defining function, if not already defined
        if type_.borrow().env.contains_key(&name) {
            bail!(TypeckError::MethodIsAlreadyDefined {
                src: self.module.source.clone(),
                span: location.span.into(),
                m: name
            })
        } else {
            type_.borrow_mut().env.insert(
                name,
                WithPublicity {
                    publicity,
                    value: Typ::Function(RcPtr::new(function)),
                },
            );
        }

        self.resolver.define(
            &location,
            &"self".into(),
            Def::Local(Typ::Custom(type_.clone())),
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

    /// Analyzes type
    fn analyze_type(
        &mut self,
        location: Address,
        name: EcoString,
        publicity: Publicity,
        params: Vec<Parameter>,
        declarations: Vec<Declaration>,
    ) {
        // inferred params vector and map
        let mut inferred_params = Vec::with_capacity(params.len());
        let mut inferred_params_map = HashMap::with_capacity(params.len());

        // inferring params
        for p in params {
            let typ = self.infer_type_annotation(p.typ);
            inferred_params.push(typ.clone());
            inferred_params_map.insert(p.name, (p.location, typ));
        }

        // construction type
        let type_ = RcPtr::new(RefCell::new(Type {
            source: location.source.clone(),
            location: location.clone(),
            name: name.clone(),
            params: inferred_params,
            env: HashMap::new(),
        }));

        // defining type, if not already defined
        self.resolver.define(
            &location,
            &name,
            Def::Module(ModDef::CustomType(WithPublicity {
                publicity,
                value: CustomType::Type(type_.clone()),
            })),
        );

        // params env start
        self.resolver.push_rib(RibKind::ConstructorParams);

        // params
        inferred_params_map.into_iter().for_each(|p| {
            self.resolver.define(&p.1.0, &p.0, Def::Local(p.1.1));
        });

        // fields env start
        self.resolver.push_rib(RibKind::Fields);

        // analyzing fields
        declarations.iter().cloned().for_each(|f| {
            if let Declaration::VarDef {
                location,
                name,
                value,
                typ,
                ..
            } = f
            {
                self.analyze_let_define(location, name, value, typ)
            }
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
        declarations.iter().cloned().for_each(|f| {
            if let Declaration::VarDef {
                publicity, name, ..
            } = f
            {
                borrowed.env.insert(
                    name.clone(),
                    WithPublicity {
                        publicity,
                        value: analyzed_fields.get(&name).unwrap().clone(),
                    },
                );
            }
        });
        drop(borrowed);

        // type env start
        self.resolver.push_rib(RibKind::Type(type_.clone()));

        // adding functions
        declarations.into_iter().for_each(|f| {
            if let Declaration::Function {
                location,
                publicity,
                name,
                params,
                body,
                typ,
            } = f
            {
                self.analyze_method(location, name, type_.clone(), publicity, params, body, typ);
            }
        });

        // type env end
        self.resolver.pop_rib();
    }

    /// Analyzes trait
    fn analyze_trait(
        &mut self,
        location: Address,
        publicity: Publicity,
        name: EcoString,
        functions: Vec<ast::TraitFunction>,
    ) {
        // construction trait
        let trait_ = RcPtr::new(Trait {
            source: self.module.source.clone(),
            location: location.clone(),
            name: name.clone(),
            functions: functions
                .into_iter()
                .map(|f| {
                    (
                        f.name.clone(),
                        RcPtr::new(Function {
                            source: self.module.source.clone(),
                            location: f.location,
                            name: f.name,
                            params: f
                                .params
                                .into_iter()
                                .map(|param| self.infer_type_annotation(param.typ))
                                .collect(),
                            ret: f.typ.map_or(Typ::Unit, |t| self.infer_type_annotation(t)),
                        }),
                    )
                })
                .collect(),
        });

        // defining type, if not already defined
        self.resolver.define(
            &location,
            &name,
            Def::Module(ModDef::CustomType(WithPublicity {
                publicity,
                value: CustomType::Trait(trait_.clone()),
            })),
        );
    }

    /// Analyzes enum
    fn analyze_enum(
        &mut self,
        location: Address,
        name: EcoString,
        publicity: Publicity,
        variants: Vec<EnumConstructor>,
    ) {
        // inferred variants
        let inferred_variants = variants
            .into_iter()
            .map(|v| EnumVariant {
                location: v.location,
                name: v.name,
                params: v
                    .params
                    .into_iter()
                    .map(|param| (param.name, self.infer_type_annotation(param.typ)))
                    .collect(),
            })
            .collect::<Vec<EnumVariant>>();

        // construction enum
        let enum_ = RcPtr::new(Enum {
            source: location.source.clone(),
            location: location.clone(),
            name: name.clone(),
            variants: inferred_variants,
        });

        // defining enum, if not already defined
        self.resolver.define(
            &location,
            &name,
            Def::Module(ModDef::CustomType(WithPublicity {
                publicity,
                value: CustomType::Enum(enum_),
            })),
        );
    }

    /// Analyzes funciton declaration
    fn analyze_function_decl(
        &mut self,
        location: Address,
        publicity: Publicity,
        name: EcoString,
        params: Vec<Parameter>,
        body: Either<Block, Expression>,
        ret_type: Option<TypePath>,
    ) {
        // inferring return type
        let ret = ret_type.map_or(Typ::Unit, |t| self.infer_type_annotation(t));

        // inferring params
        let params = params
            .into_iter()
            .map(|p| (p.name, self.infer_type_annotation(p.typ.clone())))
            .collect::<HashMap<EcoString, Typ>>();

        // creating and defining function
        let function = Function {
            source: self.module.source.clone(),
            location: location.clone(),
            name: name.clone(),
            params: params.clone().into_values().collect::<Vec<Typ>>(),
            ret: ret.clone(),
        };
        self.resolver.define(
            &location,
            &name,
            Def::Module(ModDef::Variable(WithPublicity {
                publicity,
                value: Typ::Function(RcPtr::new(function)),
            })),
        );

        // pushing new scope
        self.resolver.push_rib(RibKind::Function);

        // defining params in new scope
        params.iter().for_each(|p| {
            self.resolver
                .define(&location, p.0, Def::Local(p.1.clone()))
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

    /// Analyzes extern function declaration
    fn analyze_extern(
        &mut self,
        location: Address,
        publicity: Publicity,
        name: EcoString,
        params: Vec<Parameter>,
        ret_type: Option<TypePath>,
    ) {
        // inferring return type
        let ret = ret_type.map_or(Typ::Unit, |t| self.infer_type_annotation(t));

        // inferring params
        let params = params
            .into_iter()
            .map(|p| (p.name, self.infer_type_annotation(p.typ.clone())))
            .collect::<HashMap<EcoString, Typ>>();

        // creating and defining function
        let function = Function {
            source: self.module.source.clone(),
            location: location.clone(),
            name: name.clone(),
            params: params.clone().into_values().collect::<Vec<Typ>>(),
            ret: ret.clone(),
        };
        self.resolver.define(
            &location,
            &name,
            Def::Module(ModDef::Variable(WithPublicity {
                publicity,
                value: Typ::Function(RcPtr::new(function)),
            })),
        );
    }

    /// Analyzes define
    pub(crate) fn analyze_define(
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
                let annotated_location = annotated_path.get_location();
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
                )
            }
            None => self.resolver.define(
                &location,
                &name,
                Def::Module(ModDef::Variable(WithPublicity {
                    publicity,
                    value: inferred_value,
                })),
            ),
        }
    }

    /// Analyzes declaration
    pub fn analyze_declaration(&mut self, declaration: Declaration) {
        match declaration {
            Declaration::TypeDeclaration {
                location,
                name,
                publicity,
                constructor,
                declarations,
            } => self.analyze_type(location, name, publicity, constructor, declarations),
            Declaration::EnumDeclaration {
                location,
                name,
                publicity,
                variants,
            } => self.analyze_enum(location, name, publicity, variants),
            Declaration::ExternFunction {
                location,
                name,
                publicity,
                params,
                typ,
                ..
            } => self.analyze_extern(location, publicity, name, params, typ),
            Declaration::VarDef {
                location,
                publicity,
                name,
                value,
                typ,
            } => self.analyze_define(location, publicity, name, value, typ),
            Declaration::Function {
                location,
                publicity,
                name,
                params,
                body,
                typ,
            } => self.analyze_function_decl(location, publicity, name, params, body, typ),
            Declaration::TraitDeclaration {
                location,
                name,
                publicity,
                functions,
            } => self.analyze_trait(location, publicity, name, functions),
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
