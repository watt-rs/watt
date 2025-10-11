/// Imports
use crate::analyze::{
    errors::AnalyzeError,
    module::analyze::{CallResult, ModuleAnalyzer},
    rc_ptr::RcPtr,
    res::Res,
    resolve::{Def, ModDef},
    rib::RibKind,
    typ::{CustomType, Enum, Function, PreludeType, Typ, Type},
    warnings::AnalyzeWarning,
};
use ecow::EcoString;
use oil_ast::ast::{Publicity, TypePath};
use oil_common::{address::Address, bail, warn};
use oil_ir::ir::{IrBinaryOp, IrBlock, IrCase, IrExpression, IrParameter, IrPattern, IrUnaryOp};
use std::{cell::RefCell, collections::HashMap};

/// Expressions inferring
impl<'pkg> ModuleAnalyzer<'pkg> {
    /// Infers binary
    fn infer_binary(
        &mut self,
        location: Address,
        op: IrBinaryOp,
        left: IrExpression,
        right: IrExpression,
    ) -> Typ {
        // Inferred left and right `Typ`
        let left_typ = self.infer_expr(left);
        let right_typ = self.infer_expr(right);

        // Matching operator
        match op {
            // Concat
            IrBinaryOp::Concat => {
                // Checking prelude types
                match left_typ {
                    Typ::Prelude(PreludeType::String) => match right_typ {
                        Typ::Prelude(PreludeType::String) => Typ::Prelude(PreludeType::String),
                        _ => bail!(AnalyzeError::InvalidBinaryOp {
                            src: self.module.source.clone(),
                            span: location.span.into(),
                            a: left_typ,
                            b: right_typ,
                            op
                        }),
                    },
                    _ => bail!(AnalyzeError::InvalidBinaryOp {
                        src: self.module.source.clone(),
                        span: location.span.into(),
                        a: left_typ,
                        b: right_typ,
                        op
                    }),
                }
            }
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
                    Typ::Prelude(PreludeType::Int) => match right_typ {
                        Typ::Prelude(PreludeType::Int) => Typ::Prelude(PreludeType::Int),
                        Typ::Prelude(PreludeType::Float) => Typ::Prelude(PreludeType::Float),
                        _ => bail!(AnalyzeError::InvalidBinaryOp {
                            src: self.module.source.clone(),
                            span: location.span.into(),
                            a: left_typ,
                            b: right_typ,
                            op
                        }),
                    },
                    Typ::Prelude(PreludeType::Float) => match right_typ {
                        Typ::Prelude(PreludeType::Int) => Typ::Prelude(PreludeType::Float),
                        Typ::Prelude(PreludeType::Float) => Typ::Prelude(PreludeType::Float),
                        _ => bail!(AnalyzeError::InvalidBinaryOp {
                            src: self.module.source.clone(),
                            span: location.span.into(),
                            a: left_typ,
                            b: right_typ,
                            op
                        }),
                    },
                    _ => bail!(AnalyzeError::InvalidBinaryOp {
                        src: self.module.source.clone(),
                        span: location.span.into(),
                        a: left_typ,
                        b: right_typ,
                        op
                    }),
                }
            }
            // Logical
            IrBinaryOp::Xor | IrBinaryOp::And | IrBinaryOp::Or => {
                // Checking prelude types
                match left_typ {
                    Typ::Prelude(PreludeType::Bool) => match right_typ {
                        Typ::Prelude(PreludeType::Bool) => Typ::Prelude(PreludeType::Bool),
                        _ => bail!(AnalyzeError::InvalidBinaryOp {
                            src: self.module.source.clone(),
                            span: location.span.into(),
                            a: left_typ,
                            b: right_typ,
                            op
                        }),
                    },
                    _ => bail!(AnalyzeError::InvalidBinaryOp {
                        src: self.module.source.clone(),
                        span: location.span.into(),
                        a: left_typ,
                        b: right_typ,
                        op
                    }),
                }
            }
            // Compare
            IrBinaryOp::Le | IrBinaryOp::Lt | IrBinaryOp::Ge | IrBinaryOp::Gt => {
                // Checking prelude types
                match left_typ {
                    Typ::Prelude(PreludeType::Int) | Typ::Prelude(PreludeType::Float) => {
                        match right_typ {
                            Typ::Prelude(PreludeType::Int) | Typ::Prelude(PreludeType::Float) => {
                                Typ::Prelude(PreludeType::Bool)
                            }
                            _ => bail!(AnalyzeError::InvalidBinaryOp {
                                src: self.module.source.clone(),
                                span: location.span.into(),
                                a: left_typ,
                                b: right_typ,
                                op
                            }),
                        }
                    }
                    _ => bail!(AnalyzeError::InvalidBinaryOp {
                        src: self.module.source.clone(),
                        span: location.span.into(),
                        a: left_typ,
                        b: right_typ,
                        op
                    }),
                }
            }
            // Equality
            IrBinaryOp::Eq | IrBinaryOp::Neq => Typ::Prelude(PreludeType::Bool),
        }
    }

    /// Infers unary
    fn infer_unary(&mut self, location: Address, op: IrUnaryOp, value: IrExpression) -> Typ {
        // Inferred value `Typ`
        let inferred_value = self.infer_expr(value);

        // Checking type is prelude
        let value_typ = match &inferred_value {
            Typ::Prelude(t) => t,
            _ => bail!(AnalyzeError::InvalidUnaryOp {
                src: self.module.source.clone(),
                span: location.span.into(),
                t: inferred_value,
                op
            }),
        };

        // Matching operator
        match op {
            IrUnaryOp::Negate => match value_typ {
                PreludeType::Int => Typ::Prelude(PreludeType::Int),
                PreludeType::Float => Typ::Prelude(PreludeType::Float),
                _ => bail!(AnalyzeError::InvalidUnaryOp {
                    src: self.module.source.clone(),
                    span: location.span.into(),
                    t: inferred_value,
                    op
                }),
            },
            IrUnaryOp::Bang => match value_typ {
                PreludeType::Bool => Typ::Prelude(PreludeType::Bool),
                _ => bail!(AnalyzeError::InvalidUnaryOp {
                    src: self.module.source.clone(),
                    span: location.span.into(),
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
        field_module: EcoString,
        field_location: Address,
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
        en: RcPtr<Enum>,
        field_location: Address,
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
        ty: RcPtr<RefCell<Type>>,
        field_location: Address,
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
        &mut self,
        field_location: Address,
        container: IrExpression,
        field_name: EcoString,
    ) -> Res {
        // Inferring container
        let container_inferred = self.infer_resolution(container);
        match &container_inferred {
            // Module field access
            Res::Module(name) => {
                self.infer_module_field_access(name.clone(), field_location, field_name)
            }
            // Enum field access
            Res::Custom(CustomType::Enum(en)) => {
                self.infer_enum_field_access(en.clone(), field_location, field_name)
            }
            // Type field access
            Res::Value(typ) => match typ {
                // Custom Type
                Typ::Custom(ty) => {
                    self.infer_type_field_access(ty.clone(), field_location, field_name)
                }
                // Dyn
                Typ::Dyn => {
                    // Returning `dyn` like field type,
                    // but emitting warning
                    warn!(
                        self.package,
                        AnalyzeWarning::AccessOfDynField {
                            src: self.module.source.clone(),
                            span: field_location.span.into()
                        }
                    );
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
    pub(crate) fn infer_call(
        &mut self,
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
            Res::Custom(CustomType::Type(ty)) => {
                let borrowed = ty.borrow();
                if borrowed.params != args {
                    bail!(AnalyzeError::InvalidArgs {
                        src: self.module.source.clone(),
                        params_span: borrowed.location.span.clone().into(),
                        span: location.span.into()
                    })
                } else {
                    CallResult::FromType(Typ::Custom(ty.clone()))
                }
            }
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
                        CallResult::FromFunction(f.ret.clone(), f.clone())
                    }
                }
                // Dyn
                Typ::Dyn => {
                    // Returning `dyn` call result,
                    // but emitting warning
                    warn!(
                        self.package,
                        AnalyzeWarning::CallOfDyn {
                            src: self.module.source.clone(),
                            span: location.span.into()
                        }
                    );
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
                    CallResult::FromEnum(Typ::Enum(en.clone()))
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
    pub(crate) fn infer_type_annotation(&mut self, path: TypePath) -> Typ {
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

                match m.fields.get(&name) {
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
                        t: format!("{module}.{name}").into()
                    }),
                }
            }
            TypePath::Function {
                location,
                params,
                ret,
            } => Typ::Function(RcPtr::new(Function {
                source: self.module.source.clone(),
                location,
                name: EcoString::from("$annotated"),
                params: params
                    .into_iter()
                    .map(|p| self.infer_type_annotation(p))
                    .collect(),
                ret: self.infer_type_annotation(*ret),
            })),
        }
    }

    /// Infers resolution
    pub(crate) fn infer_resolution(&mut self, expr: IrExpression) -> Res {
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
                expr: format!("{expr:?}").into()
            }),
        }
    }

    /// Infers anonumous fn
    fn infer_anonymous_fn(
        &mut self,
        location: Address,
        params: Vec<IrParameter>,
        body: IrBlock,
        ret_type: Option<TypePath>,
    ) -> Typ {
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
            name: EcoString::from("$anonymous"),
            params: params.clone().into_values().collect::<Vec<Typ>>(),
            ret: ret.clone(),
        };

        // pushing new scope
        self.resolver.push_rib(RibKind::Function);

        // defining params in new scope
        params.iter().for_each(|p| {
            self.resolver
                .define(&self.module.source, &location, p.0, Def::Local(p.1.clone()))
        });

        // inferring body
        let block_location = &body.get_location();
        let inferred_block = self.infer_block(body);
        self.unify(&location, &ret, block_location, &inferred_block);
        self.resolver.pop_rib();

        // result
        Typ::Function(RcPtr::new(function))
    }

    /// Infers pattern matching
    pub(crate) fn infer_pattern_matching(
        &mut self,
        location: Address,
        what: IrExpression,
        cases: Vec<IrCase>,
    ) -> Typ {
        // inferring matchable
        let inferred_what = self.infer_expr(what);
        // expected return type
        let mut expected: Option<Typ> = None;
        // analyzing cases
        for case in cases {
            // Pattern scope start
            self.resolver.push_rib(RibKind::Pattern);
            // Matching pattern
            match case.pattern {
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
                                    span: case.location.span.into(),
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
                                            span: case.location.span.clone().into(),
                                            res: res.clone(),
                                            field
                                        }),
                                    }
                                });
                            }
                        }
                        _ => bail!(AnalyzeError::WrongUnwrapPattern {
                            src: self.module.source.clone(),
                            span: case.location.span.into(),
                            got: res
                        }),
                    }
                }
                IrPattern::Value(value) => {
                    let inferred_value = self.infer_expr(value);
                    if inferred_value != inferred_what {
                        bail!(AnalyzeError::TypesMissmatch {
                            src: self.module.source.clone(),
                            span: case.location.span.into(),
                            expected: inferred_what,
                            got: inferred_value
                        })
                    }
                }
                IrPattern::Range { start: _, end: _ } => todo!(),
            }
            // Analyzing body
            match &expected {
                Some(expected) => {
                    let block_location = &case.location;
                    let inferred_block = self.infer_block(case.body);
                    self.unify(&case.location, expected, block_location, &inferred_block);
                }
                None => {
                    let inferred = self.infer_block(case.body);
                    expected = Some(inferred);
                }
            }
            // Pattern scope end
            self.resolver.pop_rib();
        }
        expected.unwrap_or(Typ::Void)
    }

    /// Infers expression
    pub(crate) fn infer_expr(&mut self, expr: IrExpression) -> Typ {
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
            IrExpression::AnFn {
                location,
                params,
                body,
                typ,
            } => self.infer_anonymous_fn(location, params, body, typ),
            IrExpression::Match {
                location,
                value,
                cases,
            } => self.infer_pattern_matching(location, *value, cases),
        }
    }
}
