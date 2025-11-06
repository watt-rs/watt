/// Imports
use crate::{
    cx::module::ModuleCx,
    resolve::{
        res::Res,
        resolve::{Def, ModDef},
        rib::RibKind,
    },
    typ::{CustomType, Enum, EnumVariant, Function, Parameter, Struct, Trait, Typ, WithPublicity},
};
use ecow::EcoString;
use indexmap::IndexMap;
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use watt_ast::ast::{self, Declaration, EnumConstructor, Field, Method, Publicity, TypePath};
use watt_common::address::Address;

/// Early analyze process
impl<'pkg, 'cx> ModuleCx<'pkg, 'cx> {
    /// Early defines struct by name
    fn early_define_struct(&mut self, location: Address, publicity: Publicity, name: EcoString) {
        // Generating struct
        let strct = CustomType::Struct(Rc::new(RefCell::new(Struct {
            source: location.source.clone(),
            location: location.clone(),
            uid: self.fresh_id(),
            name: name.clone(),
            params: Vec::new(),
            env: HashMap::new(),
        })));
        // Defining struct
        self.resolver.define(
            &location,
            &name,
            Def::Module(ModDef::CustomType(WithPublicity {
                publicity,
                value: strct,
            })),
            false,
        );
    }

    /// Early defines function just by name
    fn early_define_function_decl(
        &mut self,
        location: Address,
        publicity: Publicity,
        name: EcoString,
    ) {
        // Generating function
        let function = Typ::Function(Rc::new(Function {
            source: location.source.clone(),
            location: location.clone(),
            uid: self.fresh_id(),
            name: name.clone(),
            params: Vec::new(),
            ret: Typ::Unit,
        }));
        // Defining function
        self.resolver.define(
            &location,
            &name,
            Def::Module(ModDef::Variable(WithPublicity {
                publicity,
                value: function,
            })),
            false,
        );
    }

    /// Early defines enum just by name
    fn early_define_enum(&mut self, location: Address, publicity: Publicity, name: EcoString) {
        // Generating enum
        let en = CustomType::Enum(Rc::new(Enum {
            source: location.source.clone(),
            location: location.clone(),
            uid: self.fresh_id(),
            name: name.clone(),
            variants: Vec::new(),
        }));
        // Defining enum
        self.resolver.define(
            &location,
            &name,
            Def::Module(ModDef::CustomType(WithPublicity {
                publicity,
                value: en,
            })),
            false,
        );
    }

    /// Early defines trait just by name
    fn early_define_trait(&mut self, location: Address, publicity: Publicity, name: EcoString) {
        // Generating trait
        let tr = CustomType::Trait(Rc::new(Trait {
            source: location.source.clone(),
            location: location.clone(),
            uid: self.fresh_id(),
            name: name.clone(),
            functions: HashMap::new(),
        }));
        // Defining trait
        self.resolver.define(
            &location,
            &name,
            Def::Module(ModDef::CustomType(WithPublicity {
                publicity,
                value: tr,
            })),
            false,
        );
    }

    /// Early defines extern fn just by name
    fn early_define_extern(&mut self, location: Address, publicity: Publicity, name: EcoString) {
        // Generating function
        let function = Typ::Function(Rc::new(Function {
            source: location.source.clone(),
            location: location.clone(),
            uid: self.fresh_id(),
            name: name.clone(),
            params: Vec::new(),
            ret: Typ::Unit,
        }));
        // defining function
        self.resolver.define(
            &location,
            &name,
            Def::Module(ModDef::Variable(WithPublicity {
                publicity,
                value: function,
            })),
            false,
        );
    }

    /// Early analyzes struct by inferencing it's fields and methods
    fn early_analyze_struct(
        &mut self,
        location: Address,
        name: EcoString,
        params: Vec<ast::Parameter>,
        fields: Vec<Field>,
        methods: Vec<Method>,
    ) {
        // Requesting type
        let typ = match self.resolver.resolve(&location, &name) {
            Res::Custom(ty) => match ty {
                CustomType::Struct(ty) => ty,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        };
        let mut borrowed = typ.borrow_mut();

        // Inferring params
        let params = params
            .into_iter()
            .map(|p| Parameter {
                location: p.location,
                typ: self.infer_type_annotation(p.typ),
            })
            .collect::<Vec<Parameter>>();

        // Adding fields to type env
        fields.iter().cloned().for_each(|f| {
            borrowed.env.insert(
                f.name.clone(),
                WithPublicity {
                    publicity: f.publicity,
                    value: self.infer_type_annotation(f.typ),
                },
            );
        });

        // Adding methods to type env
        methods.iter().cloned().for_each(|m| {
            // Inferring return type
            let ret = m.typ.map_or(Typ::Unit, |t| self.infer_type_annotation(t));
            self.resolver.push_rib(RibKind::Function);

            // Inferring params
            let params = m
                .params
                .iter()
                .cloned()
                .map(|p| Parameter {
                    location: p.location,
                    typ: self.infer_type_annotation(p.typ),
                })
                .collect::<Vec<Parameter>>();

            // Defining params
            params.iter().zip(m.params).for_each(|(inf_p, ast_p)| {
                self.resolver
                    .define(&location, &ast_p.name, Def::Local(inf_p.typ.clone()), false)
            });

            borrowed.env.insert(
                m.name.clone(),
                WithPublicity {
                    publicity: m.publicity,
                    value: Typ::Function(Rc::new(Function {
                        source: m.location.source.clone(),
                        location: m.location,
                        uid: self.fresh_id(),
                        name: m.name,
                        params,
                        ret,
                    })),
                },
            );
        });

        // Adding params
        borrowed.params = params
    }

    /// Early analyzes function by inferencing it's params and ret typ
    fn early_analyze_function_decl(
        &mut self,
        location: Address,
        name: EcoString,
        publicity: Publicity,
        params: Vec<ast::Parameter>,
        ret_type: Option<TypePath>,
    ) {
        // Requesting function
        let function = match self.resolver.resolve(&location, &name) {
            Res::Value(ty) => match ty {
                Typ::Function(ty) => ty,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        };

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

        // creating and redefining function
        let function = Rc::new(Function {
            source: self.module.source.clone(),
            location: location.clone(),
            uid: function.uid,
            name: name.clone(),
            params: params.into_values().collect(),
            ret: ret.clone(),
        });
        self.resolver.define(
            &location,
            &name,
            Def::Module(ModDef::Variable(WithPublicity {
                publicity,
                value: Typ::Function(function),
            })),
            true,
        );
    }

    /// Early analyzes enum by inferencing it's variants
    fn early_analyze_enum(
        &mut self,
        location: Address,
        publicity: Publicity,
        name: EcoString,
        variants: Vec<EnumConstructor>,
    ) {
        // Requesting type
        let typ = match self.resolver.resolve(&location, &name) {
            Res::Custom(ty) => match ty {
                CustomType::Enum(ty) => ty,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        };

        // Inferring variants
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

        // Constructing enum
        let enum_ = Rc::new(Enum {
            source: location.source.clone(),
            location: location.clone(),
            uid: typ.uid,
            name: name.clone(),
            variants: inferred_variants,
        });

        // Redefing enum
        self.resolver.define(
            &location,
            &name,
            Def::Module(ModDef::CustomType(WithPublicity {
                publicity,
                value: CustomType::Enum(enum_),
            })),
            true,
        );
    }

    /// Early analyzes trait by inferencing it's functions
    fn early_analyze_trait(
        &mut self,
        location: Address,
        publicity: Publicity,
        name: EcoString,
        functions: Vec<ast::TraitFunction>,
    ) {
        // Requesting trait
        let typ = match self.resolver.resolve(&location, &name) {
            Res::Custom(ty) => match ty {
                CustomType::Trait(ty) => ty,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        };

        // Constructing trait
        let trait_ = Rc::new(Trait {
            source: self.module.source.clone(),
            location: location.clone(),
            name: name.clone(),
            uid: typ.uid,
            functions: functions
                .into_iter()
                .map(|f| {
                    (
                        f.name.clone(),
                        Rc::new(Function {
                            source: self.module.source.clone(),
                            location: f.location,
                            name: f.name,
                            uid: self.fresh_id(),
                            params: f
                                .params
                                .into_iter()
                                .map(|p| Parameter {
                                    location: p.location,
                                    typ: self.infer_type_annotation(p.typ),
                                })
                                .collect(),
                            ret: f.typ.map_or(Typ::Unit, |t| self.infer_type_annotation(t)),
                        }),
                    )
                })
                .collect(),
        });

        // Redefining trait
        self.resolver.define(
            &location,
            &name,
            Def::Module(ModDef::CustomType(WithPublicity {
                publicity,
                value: CustomType::Trait(trait_.clone()),
            })),
            true,
        );
    }

    /// Early analyzes exter by inferencing it's params and ret typ
    fn early_analyze_extern(
        &mut self,
        location: Address,
        name: EcoString,
        publicity: Publicity,
        params: Vec<ast::Parameter>,
        ret_type: Option<TypePath>,
    ) {
        // Requesting function
        let function = match self.resolver.resolve(&location, &name) {
            Res::Value(ty) => match ty {
                Typ::Function(ty) => ty,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        };

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

        // creating and redefining function
        let function = Rc::new(Function {
            source: self.module.source.clone(),
            location: location.clone(),
            uid: function.uid,
            name: name.clone(),
            params: params.into_values().collect(),
            ret: ret.clone(),
        });
        self.resolver.define(
            &location,
            &name,
            Def::Module(ModDef::Variable(WithPublicity {
                publicity,
                value: Typ::Function(function),
            })),
            true,
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
                ..
            } => self.early_define_struct(location, publicity, name),
            Declaration::EnumDeclaration {
                location,
                name,
                publicity,
                ..
            } => self.early_define_enum(location, publicity, name),
            Declaration::TraitDeclaration {
                location,
                name,
                publicity,
                ..
            } => self.early_define_trait(location, publicity, name),
            Declaration::ExternFunction {
                location,
                name,
                publicity,
                ..
            } => self.early_define_extern(location, publicity, name),
            Declaration::Function {
                location,
                publicity,
                name,
                ..
            } => self.early_define_function_decl(location, publicity, name),
            _ => {}
        }
    }

    /// Early analyzes declaration
    pub(crate) fn early_analyze(&mut self, declaration: &Declaration) {
        // Matching declaration
        match declaration.clone() {
            Declaration::TypeDeclaration {
                location,
                name,
                constructor,
                fields,
                methods,
                ..
            } => self.early_analyze_struct(location, name, constructor, fields, methods),
            Declaration::EnumDeclaration {
                location,
                name,
                publicity,
                variants,
            } => self.early_analyze_enum(location, publicity, name, variants),
            Declaration::TraitDeclaration {
                location,
                name,
                publicity,
                functions,
            } => self.early_analyze_trait(location, publicity, name, functions),
            Declaration::ExternFunction {
                location,
                name,
                publicity,
                params,
                typ,
                ..
            } => self.early_analyze_extern(location, name, publicity, params, typ),
            Declaration::Function {
                location,
                publicity,
                name,
                params,
                typ,
                ..
            } => self.early_analyze_function_decl(location, name, publicity, params, typ),
            _ => {}
        }
    }
}
