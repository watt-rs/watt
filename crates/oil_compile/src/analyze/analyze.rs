/// Imports
use crate::analyze::{
    errors::AnalyzeError,
    rc_ptr::RcPtr,
    res::Res,
    resolve::{Def, ModDef, ModuleResolver},
    rib::RibKind,
    typ::{CustomType, Enum, EnumVariant, Function, Module, PreludeType, Typ, Type, WithPublicity},
    warnings::AnalyzeWarning,
};
use ecow::EcoString;
use oil_ast::ast::{Publicity, TypePath};
use oil_common::{address::Address, bail, warn};
use oil_ir::ir::{
    IrBinaryOp, IrBlock, IrDeclaration, IrEnumConstructor, IrExpression, IrFunction, IrModule,
    IrParameter, IrPattern, IrStatement, IrUnaryOp, IrVariable,
};
use std::{cell::RefCell, collections::HashMap, env::var};

/// Call result
pub enum CallResult {
    FromFunction(Typ, RcPtr<Function>),
    FromType(Typ),
    FromEnum(Typ),
    FromDyn,
}

/// Module analyzer
pub struct ModuleAnalyzer<'pkg> {
    /// Current analyzing module info
    module: &'pkg IrModule,
    module_name: &'pkg EcoString,
    /// Resolver
    resolver: ModuleResolver<'pkg>,
    /// Modules available to import
    modules: &'pkg mut HashMap<EcoString, Module>,
}

/// Implementation
impl<'pkg> ModuleAnalyzer<'pkg> {
    /// Creates new module analyzer
    pub fn new(
        module: &'pkg IrModule,
        module_name: &'pkg EcoString,
        modules: &'pkg mut HashMap<EcoString, Module>,
    ) -> Self {
        Self {
            module,
            module_name,
            resolver: ModuleResolver::new(),
            modules,
        }
    }

    /// Infers binary
    fn infer_binary(
        &self,
        location: Address,
        op: IrBinaryOp,
        left: IrExpression,
        right: IrExpression,
    ) -> Typ {
        // Inferred left and right `Typ`
        let inferred_left = self.infer_expr(left);
        let inferred_right = self.infer_expr(right);

        // Left and right `PreludeType`
        let left_typ;
        let right_typ;

        // Checking, both types are prelude
        match &inferred_left {
            Typ::Prelude(l) => {
                left_typ = l;
                match &inferred_right {
                    Typ::Prelude(r) => {
                        right_typ = r;
                    }
                    _ => bail!(AnalyzeError::InvalidBinaryOp {
                        src: self.module.source.clone(),
                        span: (location.span.start..location.span.end).into(),
                        a: inferred_left,
                        b: inferred_right,
                        op: op.into()
                    }),
                }
            }
            _ => bail!(AnalyzeError::InvalidBinaryOp {
                src: self.module.source.clone(),
                span: (location.span.start..location.span.end).into(),
                a: inferred_left,
                b: inferred_right,
                op: op.into()
            }),
        }

        // Matching operator
        match op {
            // Arithmetical
            IrBinaryOp::Add
            | IrBinaryOp::Mul
            | IrBinaryOp::Sub
            | IrBinaryOp::Div
            | IrBinaryOp::BitwiseAnd
            | IrBinaryOp::BitwiseOr
            | IrBinaryOp::Mod => {
                // Checking prelude types
                match left_typ {
                    PreludeType::Int => match right_typ {
                        PreludeType::Int => Typ::Prelude(PreludeType::Int),
                        PreludeType::Float => Typ::Prelude(PreludeType::Float),
                        _ => bail!(AnalyzeError::InvalidBinaryOp {
                            src: self.module.source.clone(),
                            span: (location.span.start..location.span.end).into(),
                            a: inferred_left,
                            b: inferred_right,
                            op
                        }),
                    },
                    PreludeType::Float => match right_typ {
                        PreludeType::Int => Typ::Prelude(PreludeType::Float),
                        PreludeType::Float => Typ::Prelude(PreludeType::Float),
                        _ => bail!(AnalyzeError::InvalidBinaryOp {
                            src: self.module.source.clone(),
                            span: (location.span.start..location.span.end).into(),
                            a: inferred_left,
                            b: inferred_right,
                            op
                        }),
                    },
                    _ => bail!(AnalyzeError::InvalidBinaryOp {
                        src: self.module.source.clone(),
                        span: (location.span.start..location.span.end).into(),
                        a: inferred_left,
                        b: inferred_right,
                        op
                    }),
                }
            }
            // Logical
            IrBinaryOp::Xor | IrBinaryOp::And | IrBinaryOp::Or => {
                // Checking prelude types
                match left_typ {
                    PreludeType::Bool => match right_typ {
                        PreludeType::Bool => Typ::Prelude(PreludeType::Bool),
                        _ => bail!(AnalyzeError::InvalidBinaryOp {
                            src: self.module.source.clone(),
                            span: (location.span.start..location.span.end).into(),
                            a: inferred_left,
                            b: inferred_right,
                            op
                        }),
                    },
                    _ => bail!(AnalyzeError::InvalidBinaryOp {
                        src: self.module.source.clone(),
                        span: (location.span.start..location.span.end).into(),
                        a: inferred_left,
                        b: inferred_right,
                        op
                    }),
                }
            }
            // Compare
            IrBinaryOp::Le | IrBinaryOp::Lt | IrBinaryOp::Ge | IrBinaryOp::Gt => {
                // Checking prelude types
                match left_typ {
                    PreludeType::Int | PreludeType::Float => match right_typ {
                        PreludeType::Int | PreludeType::Float => Typ::Prelude(PreludeType::Bool),
                        _ => bail!(AnalyzeError::InvalidBinaryOp {
                            src: self.module.source.clone(),
                            span: (location.span.start..location.span.end).into(),
                            a: inferred_left,
                            b: inferred_right,
                            op
                        }),
                    },
                    _ => bail!(AnalyzeError::InvalidBinaryOp {
                        src: self.module.source.clone(),
                        span: (location.span.start..location.span.end).into(),
                        a: inferred_left,
                        b: inferred_right,
                        op
                    }),
                }
            }
            // Equality
            IrBinaryOp::Eq | IrBinaryOp::Neq => Typ::Prelude(PreludeType::Bool),
        }
    }

    /// Infers unary
    fn infer_unary(&self, location: Address, op: IrUnaryOp, value: IrExpression) -> Typ {
        // Inferred value `Typ`
        let inferred_value = self.infer_expr(value);

        // Value `PreludeType`
        let value_typ;

        // Checking type is prelude
        match &inferred_value {
            Typ::Prelude(t) => {
                value_typ = t;
            }
            _ => bail!(AnalyzeError::InvalidUnaryOp {
                src: self.module.source.clone(),
                span: (location.span.start..location.span.end).into(),
                t: inferred_value,
                op
            }),
        }

        // Matching operator
        match op {
            IrUnaryOp::Negate => match value_typ {
                PreludeType::Int => Typ::Prelude(PreludeType::Int),
                PreludeType::Float => Typ::Prelude(PreludeType::Float),
                _ => bail!(AnalyzeError::InvalidUnaryOp {
                    src: self.module.source.clone(),
                    span: (location.span.start..location.span.end).into(),
                    t: inferred_value,
                    op
                }),
            },
            IrUnaryOp::Bang => match value_typ {
                PreludeType::Bool => Typ::Prelude(PreludeType::Bool),
                _ => bail!(AnalyzeError::InvalidUnaryOp {
                    src: self.module.source.clone(),
                    span: (location.span.start..location.span.end).into(),
                    t: inferred_value,
                    op
                }),
            },
        }
    }

    /// Infers get
    fn infer_get(&self, location: Address, name: EcoString) -> Res {
        self.resolver.resolve(&self.module.source, &location, &name)
    }

    /// Infers module field access
    fn infer_module_field_access(
        &self,
        field_location: Address,
        field_module: EcoString,
        field_name: EcoString,
    ) -> Res {
        // Getting module
        match self.resolver.imported_modules.get(&field_module) {
            // Getting module
            Some(module) => match module.fields.get(&field_name) {
                // If field exists
                // checking it's publicity
                Some(def) => match def {
                    ModDef::CustomType(ty) => {
                        match ty.publicity {
                            // If field is public, we resolved field
                            Publicity::Public => Res::Custom(ty.value.clone()),
                            // Else, raising `module field is private`
                            _ => bail!(AnalyzeError::ModuleFieldIsPrivate {
                                src: self.module.source.clone(),
                                span: field_location.span.into(),
                                name: field_name
                            }),
                        }
                    }
                    ModDef::Variable(var) => {
                        match var.publicity {
                            // If field is public, we resolved field
                            Publicity::Public => Res::Value(var.value.clone()),
                            // Else, raising `module field is private`
                            _ => bail!(AnalyzeError::ModuleFieldIsPrivate {
                                src: self.module.source.clone(),
                                span: field_location.span.into(),
                                name: field_name
                            }),
                        }
                    }
                },
                // Else, raising `module field is not defined`
                None => bail!(AnalyzeError::ModuleFieldIsNotDefined {
                    src: self.module.source.clone(),
                    span: field_location.span.into(),
                    m: field_module,
                    field: field_name
                }),
            },
            // If module is not defined
            None => bail!(AnalyzeError::ModuleIsNotDefined { m: field_module }),
        }
    }

    /// Infers enum field access
    fn infer_enum_field_access(
        &self,
        field_location: Address,
        en: RcPtr<Enum>,
        field_name: EcoString,
    ) -> Res {
        // Finding variant
        match en.variants.iter().find(|var| var.name == field_name) {
            Some(variant) => Res::Variant(en.clone(), variant.clone()),
            None => bail!(AnalyzeError::EnumVariantIsNotDefined {
                src: self.module.source.clone(),
                span: field_location.span.into(),
                e: en.name.clone(),
                variant: field_name
            }),
        }
    }

    /// Infers type field access
    fn infer_type_field_access(
        &self,
        field_location: Address,
        ty: RcPtr<RefCell<Type>>,
        field_name: EcoString,
    ) -> Res {
        // Finding field
        let borrowed = ty.borrow();
        match borrowed.env.get(&field_name) {
            Some(typ) => match typ.publicity {
                // Checking publicity
                Publicity::Public => Res::Value(typ.value.clone()),
                // Checking environments stack contains type
                _ => match self.resolver.contains_type_rib() {
                    // If type is same
                    Some(t) => {
                        if *t == ty {
                            Res::Value(typ.value.clone())
                        } else {
                            bail!(AnalyzeError::FieldIsPrivate {
                                src: self.module.source.clone(),
                                span: field_location.span.into(),
                                t: borrowed.name.clone(),
                                field: field_name
                            });
                        }
                    }
                    // Else
                    None => bail!(AnalyzeError::FieldIsPrivate {
                        src: self.module.source.clone(),
                        span: field_location.span.into(),
                        t: borrowed.name.clone(),
                        field: field_name
                    }),
                },
            },
            None => bail!(AnalyzeError::FieldIsNotDefined {
                src: self.module.source.clone(),
                span: field_location.span.into(),
                t: borrowed.name.clone(),
                field: field_name
            }),
        }
    }

    /// Infers access
    fn infer_field_access(
        &self,
        field_location: Address,
        container: IrExpression,
        field_name: EcoString,
    ) -> Res {
        // Inferring container
        let container_inferred = self.infer_resolution(container);
        match &container_inferred {
            // Module field access
            Res::Module(name) => {
                self.infer_module_field_access(field_location, name.clone(), field_name)
            }
            // Enum field access
            Res::Custom(custom) => match custom {
                CustomType::Enum(en) => {
                    self.infer_enum_field_access(field_location, en.clone(), field_name)
                }
                _ => bail!(AnalyzeError::CouldNotResolveFieldsIn {
                    src: self.module.source.clone(),
                    span: field_location.span.into(),
                    res: container_inferred
                }),
            },
            // Type field access
            Res::Value(typ) => match typ {
                // Custom Type
                Typ::Custom(t) => {
                    self.infer_type_field_access(field_location, t.clone(), field_name)
                }
                // Dyn
                Typ::Dyn => {
                    // Returning `dyn` like field type,
                    // but emitting warning
                    warn!(AnalyzeWarning::AccessOfDynField {
                        src: self.module.source.clone(),
                        span: field_location.span.into()
                    });
                    Res::Value(Typ::Dyn)
                }
                // Else
                _ => bail!(AnalyzeError::CouldNotResolveFieldsIn {
                    src: self.module.source.clone(),
                    span: field_location.span.into(),
                    res: container_inferred
                }),
            },
            // Else
            _ => bail!(AnalyzeError::CouldNotResolveFieldsIn {
                src: self.module.source.clone(),
                span: field_location.span.into(),
                res: container_inferred
            }),
        }
    }

    /// Infers call
    fn infer_call(
        &self,
        location: Address,
        what: IrExpression,
        args: Vec<IrExpression>,
    ) -> CallResult {
        let function = self.infer_resolution(what);
        let args = args
            .iter()
            .map(|a| self.infer_expr(a.clone()))
            .collect::<Vec<Typ>>();
        match &function {
            // Custom type
            Res::Custom(t) => match t {
                CustomType::Type(ty) => {
                    let borrowed = ty.borrow();
                    if borrowed.params != args {
                        bail!(AnalyzeError::InvalidArgs {
                            src: self.module.source.clone(),
                            params_span: borrowed.location.span.clone().into(),
                            span: location.span.into()
                        })
                    } else {
                        return CallResult::FromType(Typ::Custom(ty.clone()));
                    }
                }
                _ => bail!(AnalyzeError::CouldNotCall {
                    src: self.module.source.clone(),
                    span: location.span.into(),
                    res: function
                }),
            },
            // Value
            Res::Value(t) => match t {
                // Function
                Typ::Function(f) => {
                    if f.params != args {
                        bail!(AnalyzeError::InvalidArgs {
                            src: self.module.source.clone(),
                            params_span: f.location.span.clone().into(),
                            span: location.span.into()
                        })
                    } else {
                        return CallResult::FromFunction(f.ret.clone(), f.clone());
                    }
                }
                // Dyn
                Typ::Dyn => {
                    // Returning `dyn` call result,
                    // but emitting warning
                    warn!(AnalyzeWarning::CallOfDyn {
                        src: self.module.source.clone(),
                        span: location.span.into()
                    });
                    CallResult::FromDyn
                }
                // Else
                _ => bail!(AnalyzeError::CouldNotCall {
                    src: self.module.source.clone(),
                    span: location.span.into(),
                    res: function
                }),
            },
            // Variant
            Res::Variant(en, variant) => {
                if variant.params.values().cloned().collect::<Vec<Typ>>() != args {
                    bail!(AnalyzeError::InvalidArgs {
                        src: self.module.source.clone(),
                        params_span: variant.location.span.clone().into(),
                        span: location.span.into()
                    })
                } else {
                    return CallResult::FromEnum(Typ::Enum(en.clone()));
                }
            }
            _ => bail!(AnalyzeError::CouldNotCall {
                src: self.module.source.clone(),
                span: location.span.into(),
                res: function
            }),
        }
    }

    /// Infers type annotation
    fn infer_type_annotation(&self, path: TypePath) -> Typ {
        match path {
            TypePath::Local { location, name } => match name.as_str() {
                "int" => Typ::Prelude(PreludeType::Int),
                "float" => Typ::Prelude(PreludeType::Float),
                "bool" => Typ::Prelude(PreludeType::Bool),
                "string" => Typ::Prelude(PreludeType::String),
                "dyn" => Typ::Dyn,
                _ => match self
                    .resolver
                    .resolve_type(&name, &self.module.source, &location)
                {
                    CustomType::Enum(en) => Typ::Enum(en.clone()),
                    CustomType::Type(ty) => Typ::Custom(ty.clone()),
                },
            },
            TypePath::Module {
                location,
                module,
                name,
            } => {
                let m = self.resolver.resolve_module(&module);
                let typ = match m.fields.get(&name) {
                    Some(field) => match field {
                        ModDef::CustomType(t) => {
                            if t.publicity != Publicity::Private {
                                match &t.value {
                                    CustomType::Enum(en) => Typ::Enum(en.clone()),
                                    CustomType::Type(ty) => Typ::Custom(ty.clone()),
                                }
                            } else {
                                bail!(AnalyzeError::TypeIsPrivate {
                                    src: self.module.source.clone(),
                                    span: location.span.into(),
                                    t: t.value.clone()
                                })
                            }
                        }
                        ModDef::Variable(_) => bail!(AnalyzeError::CouldNotUseValueAsType {
                            src: self.module.source.clone(),
                            span: location.clone().span.into(),
                            v: name
                        }),
                    },
                    None => bail!(AnalyzeError::TypeIsNotDefined {
                        src: self.module.source.clone(),
                        span: location.span.into(),
                        t: format!("{}.{}", module, name).into()
                    }),
                };
                typ
            }
        }
    }

    /// Infers resolution
    fn infer_resolution(&self, expr: IrExpression) -> Res {
        match expr {
            IrExpression::Get { location, name } => self.infer_get(location, name),
            IrExpression::FieldAccess {
                location,
                container,
                name,
            } => self.infer_field_access(location, *container, name),
            IrExpression::Call {
                location,
                what,
                args,
            } => match self.infer_call(location.clone(), *what, args) {
                CallResult::FromFunction(typ, function) => {
                    if typ == Typ::Void {
                        bail!(AnalyzeError::CallExprReturnTypeIsVoid {
                            fn_src: function.source.clone(),
                            definition_span: function.location.span.clone().into(),
                            call_src: self.module.source.clone(),
                            span: location.span.into()
                        })
                    } else {
                        Res::Value(typ)
                    }
                }
                CallResult::FromType(t) => Res::Value(t),
                CallResult::FromEnum(e) => Res::Value(e),
                CallResult::FromDyn => Res::Value(Typ::Dyn),
            },
            expr => bail!(AnalyzeError::UnexpectedExprInResolution {
                expr: format!("{:?}", expr).into()
            }),
        }
    }

    /// Infers expression
    fn infer_expr(&self, expr: IrExpression) -> Typ {
        match expr {
            IrExpression::Float { .. } => Typ::Prelude(PreludeType::Float),
            IrExpression::Int { .. } => Typ::Prelude(PreludeType::Int),
            IrExpression::String { .. } => Typ::Prelude(PreludeType::String),
            IrExpression::Bool { .. } => Typ::Prelude(PreludeType::Bool),
            IrExpression::Bin {
                location,
                left,
                right,
                op,
            } => self.infer_binary(location, op, *left, *right),
            IrExpression::Unary {
                location,
                value,
                op,
            } => self.infer_unary(location, op, *value),
            IrExpression::Get { location, name } => self
                .infer_get(location.clone(), name)
                .unwrap_typ(&self.module.source, &location),
            IrExpression::FieldAccess {
                location,
                container,
                name,
            } => self
                .infer_field_access(location.clone(), *container, name)
                .unwrap_typ(&self.module.source, &location),
            IrExpression::Call {
                location,
                what,
                args,
            } => match self.infer_call(location.clone(), *what, args) {
                CallResult::FromFunction(typ, function) => {
                    if typ == Typ::Void {
                        bail!(AnalyzeError::CallExprReturnTypeIsVoid {
                            fn_src: function.source.clone(),
                            definition_span: function.location.span.clone().into(),
                            call_src: self.module.source.clone(),
                            span: location.span.into()
                        })
                    } else {
                        typ
                    }
                }
                CallResult::FromType(t) => t,
                CallResult::FromEnum(e) => e,
                CallResult::FromDyn => Typ::Dyn,
            },
            IrExpression::Range { .. } => todo!(),
            IrExpression::Match { .. } => todo!(),
        }
    }

    /// Analyzes block
    fn analyze_block(&mut self, block: IrBlock) {
        for statement in block.nodes {
            self.analyze_statement(statement);
        }
    }

    /// Analyzes define
    fn analyze_define(
        &mut self,
        location: Address,
        name: EcoString,
        value: IrExpression,
        typ: Option<TypePath>,
    ) {
        let inferred_value = self.infer_expr(value);
        match typ {
            Some(annotated_path) => {
                let annotated = self.infer_type_annotation(annotated_path);
                if inferred_value == annotated {
                    // defining by annotated type, because:
                    //
                    // 1. we can get
                    // non-dyn value, but annotation will be `dyn`
                    //
                    // 2. we can get
                    // dyn value, but annotation will not be `dyn`
                    //
                    self.resolver.define(
                        &self.module.source,
                        &location,
                        &name,
                        Def::Local(annotated),
                    )
                } else {
                    bail!(AnalyzeError::MissmatchedTypeAnnotation {
                        src: self.module.source.clone(),
                        span: location.span.into(),
                        expected: annotated,
                        got: inferred_value
                    })
                }
            }
            None => self.resolver.define(
                &self.module.source,
                &location,
                &name,
                Def::Local(inferred_value),
            ),
        }
    }

    /// Analyzes assignment
    fn analyze_assignment(&mut self, location: Address, what: IrExpression, value: IrExpression) {
        let inferred_what = self.infer_expr(what);
        let inferred_value = self.infer_expr(value);
        if inferred_what != inferred_value {
            bail!(AnalyzeError::TypesMissmatch {
                src: self.module.source.clone(),
                span: location.span.into(),
                expected: inferred_what,
                got: inferred_value
            })
        }
    }

    /// Analyzes funciton statement
    fn analyze_function_stmt(
        &mut self,
        location: Address,
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
            Def::Local(Typ::Function(RcPtr::new(function))),
        );

        // pushing new scope
        self.resolver.push_rib(RibKind::Function(ret.clone()));

        // defining params in new scope
        params.iter().for_each(|p| {
            self.resolver
                .define(&self.module.source, &location, p.0, Def::Local(p.1.clone()))
        });

        // inferring body
        self.analyze_block(body);
        self.resolver.pop_rib();
    }

    /// Analyzes pattern matching
    fn analyze_pattern_matching(
        &mut self,
        location: Address,
        what: IrExpression,
        patterns: Vec<(IrPattern, IrBlock)>,
    ) {
        // inferring matchable
        let inferred_what = self.infer_expr(what);
        // analyzing patterns
        for pattern in patterns {
            // Pattern scope start
            self.resolver.push_rib(RibKind::Pattern);
            // Matching pattern
            match pattern.0 {
                IrPattern::Unwrap { en, fields } => {
                    // inferring resolution, and checking
                    // that is a enum variant
                    let res = self.infer_resolution(en);
                    match &res {
                        Res::Variant(en, variant) => {
                            // If types aren't equal
                            let en_typ = Typ::Enum(en.clone());
                            if inferred_what != en_typ {
                                bail!(AnalyzeError::TypesMissmatch {
                                    src: self.module.source.clone(),
                                    span: location.span.into(),
                                    expected: en_typ,
                                    got: inferred_what.clone()
                                });
                            }
                            // If types equal, checking fields existence
                            else {
                                fields.into_iter().for_each(|field| {
                                    match variant.params.get(&field) {
                                        // Defining field with it's type, if it exists
                                        Some(typ) => {
                                            self.resolver.define(
                                                &self.module.source,
                                                &location,
                                                &field,
                                                Def::Local(typ.clone()),
                                            );
                                        }
                                        None => bail!(AnalyzeError::EnumVariantFieldIsNotDefined {
                                            src: self.module.source.clone(),
                                            span: location.span.clone().into(),
                                            res: res.clone(),
                                            field
                                        }),
                                    }
                                });
                            }
                        }
                        _ => bail!(AnalyzeError::WrongUnwrapPattern {
                            src: self.module.source.clone(),
                            span: location.span.into(),
                            got: res
                        }),
                    }
                }
                IrPattern::Value(value) => {
                    let inferred_value = self.infer_expr(value);
                    if inferred_value != inferred_what {
                        bail!(AnalyzeError::TypesMissmatch {
                            src: self.module.source.clone(),
                            span: location.span.into(),
                            expected: inferred_what,
                            got: inferred_value
                        })
                    }
                }
                IrPattern::Range { start, end } => todo!(),
            }
            // Analyzing body
            self.analyze_block(pattern.1);
            // Pattern scope end
            self.resolver.pop_rib();
        }
    }

    /// Analyzes statement
    fn analyze_statement(&mut self, statement: IrStatement) {
        match statement {
            IrStatement::If {
                location,
                logical,
                body,
                elseif,
            } => {
                // pushing rib
                self.resolver.push_rib(RibKind::Conditional);
                // inferring logical
                let inferred_logical = self.infer_expr(logical);
                match inferred_logical {
                    Typ::Prelude(prelude) => match prelude {
                        PreludeType::Bool => {}
                        _ => bail!(AnalyzeError::ExpectedLogicalInIf {
                            src: self.module.source.clone(),
                            span: location.span.into()
                        }),
                    },
                    _ => bail!(AnalyzeError::ExpectedLogicalInIf {
                        src: self.module.source.clone(),
                        span: location.span.into()
                    }),
                }
                // analyzing block
                self.analyze_block(body);
                // popping rib
                self.resolver.pop_rib();
                // analyzing elseif
                match elseif {
                    Some(elseif) => self.analyze_statement(*elseif),
                    None => {}
                }
            }
            IrStatement::While {
                location,
                logical,
                body,
            } => {
                // pushing rib
                self.resolver.push_rib(RibKind::Loop);
                // inferring logical
                let inferred_logical = self.infer_expr(logical);
                match inferred_logical {
                    Typ::Prelude(prelude) => match prelude {
                        PreludeType::Bool => {}
                        _ => bail!(AnalyzeError::ExpectedLogicalInWhile {
                            src: self.module.source.clone(),
                            span: location.span.into()
                        }),
                    },
                    _ => bail!(AnalyzeError::ExpectedLogicalInWhile {
                        src: self.module.source.clone(),
                        span: location.span.into()
                    }),
                }
                // analyzing block
                self.analyze_block(body);
                // popping rib
                self.resolver.pop_rib();
            }
            IrStatement::Define {
                location,
                name,
                value,
                typ,
            } => self.analyze_define(location, name, value, typ),
            IrStatement::Assign {
                location,
                what,
                value,
            } => self.analyze_assignment(location, *what, *value),
            IrStatement::Call {
                location,
                what,
                args,
            } => {
                self.infer_call(location, *what, args);
            }
            IrStatement::Fn {
                location,
                name,
                params,
                body,
                typ,
            } => self.analyze_function_stmt(location, name, params, body, typ),
            IrStatement::Break { location } => {
                if !self.resolver.contains_rib(RibKind::Loop) {
                    bail!(AnalyzeError::BreakWithoutLoop {
                        src: self.module.source.clone(),
                        span: location.span.into(),
                    })
                }
            }
            IrStatement::Continue { location } => {
                if !self.resolver.contains_rib(RibKind::Loop) {
                    bail!(AnalyzeError::ContinueWithoutLoop {
                        src: self.module.source.clone(),
                        span: location.span.into(),
                    })
                }
            }
            IrStatement::Return { location, value } => {
                let inferred_value = self.infer_expr(value);
                match self.resolver.contains_fn_rib() {
                    Some(ret_type) => {
                        if &inferred_value != ret_type {
                            bail!(AnalyzeError::WrongReturnType {
                                src: self.module.source.clone(),
                                span: location.span.into(),
                                expected: ret_type.clone(),
                                got: inferred_value
                            })
                        }
                    }
                    None => bail!(AnalyzeError::ReturnWithoutFunction {
                        src: self.module.source.clone(),
                        span: location.span.into(),
                    }),
                }
            }
            IrStatement::For { .. } => todo!(),
            IrStatement::Match {
                location,
                value,
                patterns,
            } => self.analyze_pattern_matching(location, value, patterns),
        }
    }

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
        self.resolver.push_rib(RibKind::Function(ret.clone()));

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
        self.analyze_block(body);
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
        self.resolver.push_rib(RibKind::Function(ret.clone()));

        // defining params in new scope
        params.iter().for_each(|p| {
            self.resolver
                .define(&self.module.source, &location, p.0, Def::Local(p.1.clone()))
        });

        // inferring body
        self.analyze_block(body);
        self.resolver.pop_rib();
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
        }
    }

    /// Performs analyze of module
    pub fn analyze(&mut self) -> Module {
        for definition in self.module.clone().definitions {
            self.analyze_declaration(definition)
        }
        Module {
            source: self.module.source.clone(),
            name: self.module_name.clone(),
            fields: self.resolver.collect(),
        }
    }
}
