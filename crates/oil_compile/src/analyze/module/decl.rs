/// Imports
use crate::analyze::{
    errors::AnalyzeError,
    module::analyze::ModuleAnalyzer,
    rc_ptr::RcPtr,
    resolve::{Def, ModDef},
    rib::RibKind,
    typ::{CustomType, Enum, EnumVariant, Function, Typ, Type, WithPublicity},
};
use ecow::EcoString;
use oil_ast::ast::{Publicity, TypePath};
use oil_common::{address::Address, bail};
use oil_ir::ir::{
    IrBlock, IrDeclaration, IrDependency, IrDependencyKind, IrEnumConstructor, IrFunction,
    IrParameter, IrVariable,
};
use std::{cell::RefCell, collections::HashMap};

/// Declaraton analyze
impl<'pkg> ModuleAnalyzer<'pkg> {
    /// Analyzes method
    fn analyze_method(
        &mut self,
        location: Address,
        name: EcoString,
        type_: RcPtr<RefCell<Type>>,
        publicity: Publicity,
        params: Vec<IrParameter>,
        body: IrBlock,
        ret_type: Option<TypePath>,
    ) {
        // inferring return type
        let ret = ret_type.map_or(Typ::Void, |t| self.infer_type_annotation(t));
        self.resolver.push_rib(RibKind::Function);

        // inferring params
        let params = params
            .into_iter()
            .map(|p| (p.name, self.infer_type_annotation(p.typ.clone())))
            .collect::<HashMap<EcoString, Typ>>();

        params.iter().for_each(|p| {
            self.resolver
                .define(&self.module.source, &location, p.0, Def::Local(p.1.clone()))
        });

        self.resolver.define(
            &self.module.source,
            &location,
            &"self".into(),
            Def::Local(Typ::Custom(type_.clone())),
        );

        // inferring body
        let block_location = body.get_location();
        let inferred_block = self.infer_block(body);
        self.unify(&location, &ret, &block_location, &inferred_block);
        self.resolver.pop_rib();

        // creating and defining function
        let function = Function {
            source: self.module.source.clone(),
            location: location.clone(),
            name: name.clone(),
            params: params.into_iter().map(|(_, v)| v).collect::<Vec<Typ>>(),
            ret,
        };

        // defining function, if not already defined
        if let Some(_) = type_.borrow().env.get(&name).cloned() {
            bail!(AnalyzeError::MethodIsAlreadyDefined {
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
    }

    /// Analyzes type
    fn analyze_type(
        &mut self,
        location: Address,
        name: EcoString,
        publicity: Publicity,
        params: Vec<IrParameter>,
        fields: Vec<IrVariable>,
        functions: Vec<IrFunction>,
    ) {
        // inferred params
        let inferred_params = params
            .into_iter()
            .map(|p| (p.name, (p.location, self.infer_type_annotation(p.typ))))
            .collect::<HashMap<EcoString, (Address, Typ)>>();

        // construction type
        let type_ = RcPtr::new(RefCell::new(Type {
            source: self.module.source.clone(),
            location: location.clone(),
            name: name.clone(),
            params: inferred_params.iter().map(|p| p.1.1.clone()).collect(),
            env: HashMap::new(),
        }));

        // params env start
        self.resolver.push_rib(RibKind::ConstructorParams);

        // params
        inferred_params.into_iter().for_each(|p| {
            self.resolver
                .define(&self.module.source, &p.1.0, &p.0, Def::Local(p.1.1));
        });

        // fields env start
        self.resolver.push_rib(RibKind::Fields);

        // fields
        fields.clone().into_iter().for_each(|f| {
            self.analyze_define(f.location, f.name, f.value, f.typ);
        });

        // fields env end
        let analyzed_fields = match self.resolver.pop_rib() {
            Some(fields) => fields.1,
            None => bail!(AnalyzeError::EnvironmentsStackIsEmpty),
        };

        // params env end
        self.resolver.pop_rib();

        // adding fields to type env
        let mut borrowed = type_.borrow_mut();
        fields.into_iter().for_each(|f| {
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
        self.resolver.push_rib(RibKind::Type(type_.clone()));

        // adding functions
        functions.into_iter().for_each(|f| {
            self.analyze_method(
                f.location,
                f.name,
                type_.clone(),
                f.publicity,
                f.params,
                f.body,
                f.typ,
            );
        });

        // type env end
        self.resolver.pop_rib();

        // defining type, if not already defined
        self.resolver.define(
            &self.module.source,
            &location,
            &name,
            Def::Module(ModDef::CustomType(WithPublicity {
                publicity,
                value: CustomType::Type(type_),
            })),
        );
    }

    /// Analyzes enum
    fn analyze_enum(
        &mut self,
        location: Address,
        name: EcoString,
        publicity: Publicity,
        variants: Vec<IrEnumConstructor>,
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
            source: self.module.source.clone(),
            location: location.clone(),
            name: name.clone(),
            variants: inferred_variants,
        });

        // defining enum, if not already defined
        self.resolver.define(
            &self.module.source,
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
        params: Vec<IrParameter>,
        body: IrBlock,
        ret_type: Option<TypePath>,
    ) {
        // inferring return type
        let ret = ret_type.map_or(Typ::Void, |t| self.infer_type_annotation(t));

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
            params: params
                .clone()
                .into_iter()
                .map(|(_, v)| v)
                .collect::<Vec<Typ>>(),
            ret: ret.clone(),
        };
        self.resolver.define(
            &self.module.source.clone(),
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
                .define(&self.module.source, &location, p.0, Def::Local(p.1.clone()))
        });

        // inferring body
        let block_location = body.get_location();
        let inferred_block = self.infer_block(body);
        self.unify(&location, &ret, &block_location, &inferred_block);
        self.resolver.pop_rib();
    }

    /// Analyzes extern function declaration
    fn analyze_extern(
        &mut self,
        location: Address,
        publicity: Publicity,
        name: EcoString,
        params: Vec<IrParameter>,
        ret_type: Option<TypePath>,
    ) {
        // inferring return type
        let ret = ret_type.map_or(Typ::Void, |t| self.infer_type_annotation(t));

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
            params: params
                .clone()
                .into_iter()
                .map(|(_, v)| v)
                .collect::<Vec<Typ>>(),
            ret: ret.clone(),
        };
        self.resolver.define(
            &self.module.source.clone(),
            &location,
            &name,
            Def::Module(ModDef::Variable(WithPublicity {
                publicity,
                value: Typ::Function(RcPtr::new(function)),
            })),
        );
    }

    /// Analyzes declaration
    pub fn analyze_declaration(&mut self, declaration: IrDeclaration) {
        match declaration {
            IrDeclaration::Function(ir_function) => self.analyze_function_decl(
                ir_function.location,
                ir_function.publicity,
                ir_function.name,
                ir_function.params,
                ir_function.body,
                ir_function.typ,
            ),
            IrDeclaration::Variable(ir_variable) => self.analyze_define(
                ir_variable.location,
                ir_variable.name,
                ir_variable.value,
                ir_variable.typ,
            ),
            IrDeclaration::Type(ir_type) => self.analyze_type(
                ir_type.location,
                ir_type.name,
                ir_type.publicity,
                ir_type.constructor,
                ir_type.fields,
                ir_type.functions,
            ),
            IrDeclaration::Enum(ir_enum) => self.analyze_enum(
                ir_enum.location,
                ir_enum.name,
                ir_enum.publicity,
                ir_enum.variants,
            ),
            IrDeclaration::Extern(ir_extern) => self.analyze_extern(
                ir_extern.location,
                ir_extern.publicity,
                ir_extern.name,
                ir_extern.params,
                ir_extern.typ,
            ),
        }
    }

    /// Performs import
    pub fn perform_import(&mut self, import: IrDependency) {
        match self.modules.get(&import.path) {
            Some(module) => match import.kind {
                IrDependencyKind::AsName(name) => self.resolver.import_as(
                    &self.module.source,
                    &import.location,
                    name,
                    module.clone(),
                ),
                IrDependencyKind::ForNames(names) => self.resolver.import_for(
                    &self.module.source,
                    &import.location,
                    names,
                    module.clone(),
                ),
            },
            None => bail!(AnalyzeError::ImportOfUnknownModule {
                src: self.module.source.clone(),
                span: import.location.span.into(),
                m: import.path
            }),
        };
    }
}
