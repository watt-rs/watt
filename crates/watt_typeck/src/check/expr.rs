/// Imports
use crate::{
    cx::module::ModuleCx,
    errors::TypeckError,
    ex::ExMatchCx,
    resolve::{
        res::Res,
        resolve::{Def, ModDef},
        rib::RibKind,
    },
    typ::{CustomType, Enum, Function, PreludeType, Typ, Type},
    unify::Equation,
    utils::CallResult,
    warnings::TypeckWarning,
};
use ecow::EcoString;
use std::{cell::RefCell, collections::HashMap};
use watt_ast::ast::{
    BinaryOp, Block, Case, ElseBranch, Expression, Parameter, Pattern, Publicity, TypePath, UnaryOp,
};
use watt_common::{address::Address, bail, rc_ptr::RcPtr, warn};

/// Expressions inferring
impl<'pkg, 'cx> ModuleCx<'pkg, 'cx> {
    /// Infers binary
    fn infer_binary(
        &mut self,
        location: Address,
        op: BinaryOp,
        left: Expression,
        right: Expression,
    ) -> Typ {
        // Inferred left and right `Typ`
        let left_typ = self.infer_expr(left);
        let right_typ = self.infer_expr(right);

        // Matching operator
        match op {
            // Concat
            BinaryOp::Concat => {
                // Checking prelude types
                match left_typ {
                    Typ::Prelude(PreludeType::String) => match right_typ {
                        Typ::Prelude(PreludeType::String) => Typ::Prelude(PreludeType::String),
                        _ => bail!(TypeckError::InvalidBinaryOp {
                            src: self.module.source.clone(),
                            span: location.span.into(),
                            a: left_typ,
                            b: right_typ,
                            op
                        }),
                    },
                    _ => bail!(TypeckError::InvalidBinaryOp {
                        src: self.module.source.clone(),
                        span: location.span.into(),
                        a: left_typ,
                        b: right_typ,
                        op
                    }),
                }
            }
            // Arithmetical
            BinaryOp::Add
            | BinaryOp::Sub
            | BinaryOp::Mul
            | BinaryOp::Div
            | BinaryOp::BitwiseAnd
            | BinaryOp::BitwiseOr
            | BinaryOp::Mod => {
                // Checking prelude types
                match left_typ {
                    Typ::Prelude(PreludeType::Int) => match right_typ {
                        Typ::Prelude(PreludeType::Int) => Typ::Prelude(PreludeType::Int),
                        Typ::Prelude(PreludeType::Float) => Typ::Prelude(PreludeType::Float),
                        _ => bail!(TypeckError::InvalidBinaryOp {
                            src: self.module.source.clone(),
                            span: location.span.into(),
                            a: left_typ,
                            b: right_typ,
                            op: op.into()
                        }),
                    },
                    Typ::Prelude(PreludeType::Float) => match right_typ {
                        Typ::Prelude(PreludeType::Int) => Typ::Prelude(PreludeType::Float),
                        Typ::Prelude(PreludeType::Float) => Typ::Prelude(PreludeType::Float),
                        _ => bail!(TypeckError::InvalidBinaryOp {
                            src: self.module.source.clone(),
                            span: location.span.into(),
                            a: left_typ,
                            b: right_typ,
                            op: op.into()
                        }),
                    },
                    _ => bail!(TypeckError::InvalidBinaryOp {
                        src: self.module.source.clone(),
                        span: location.span.into(),
                        a: left_typ,
                        b: right_typ,
                        op: op.into()
                    }),
                }
            }
            // Logical
            BinaryOp::Xor | BinaryOp::And | BinaryOp::Or => {
                // Checking prelude types
                match left_typ {
                    Typ::Prelude(PreludeType::Bool) => match right_typ {
                        Typ::Prelude(PreludeType::Bool) => Typ::Prelude(PreludeType::Bool),
                        _ => bail!(TypeckError::InvalidBinaryOp {
                            src: self.module.source.clone(),
                            span: location.span.into(),
                            a: left_typ,
                            b: right_typ,
                            op: op.into()
                        }),
                    },
                    _ => bail!(TypeckError::InvalidBinaryOp {
                        src: self.module.source.clone(),
                        span: location.span.into(),
                        a: left_typ,
                        b: right_typ,
                        op: op.into()
                    }),
                }
            }
            // Compare
            BinaryOp::Ge | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Lt => {
                // Checking prelude types
                match left_typ {
                    Typ::Prelude(PreludeType::Int) | Typ::Prelude(PreludeType::Float) => {
                        match right_typ {
                            Typ::Prelude(PreludeType::Int) | Typ::Prelude(PreludeType::Float) => {
                                Typ::Prelude(PreludeType::Bool)
                            }
                            _ => bail!(TypeckError::InvalidBinaryOp {
                                src: self.module.source.clone(),
                                span: location.span.into(),
                                a: left_typ,
                                b: right_typ,
                                op: op.into()
                            }),
                        }
                    }
                    _ => bail!(TypeckError::InvalidBinaryOp {
                        src: self.module.source.clone(),
                        span: location.span.into(),
                        a: left_typ,
                        b: right_typ,
                        op: op.into()
                    }),
                }
            }
            // Equality
            BinaryOp::Eq | BinaryOp::NotEq => Typ::Prelude(PreludeType::Bool),
        }
    }

    /// Infers unary
    fn infer_unary(&mut self, location: Address, op: UnaryOp, value: Expression) -> Typ {
        // Inferred value `Typ`
        let inferred_value = self.infer_expr(value);

        // Checking type is prelude
        let value_typ = match &inferred_value {
            Typ::Prelude(t) => t,
            _ => bail!(TypeckError::InvalidUnaryOp {
                src: self.module.source.clone(),
                span: location.span.into(),
                t: inferred_value,
                op
            }),
        };

        // Matching operator
        match op {
            // Negate `-`
            UnaryOp::Neg => match value_typ {
                PreludeType::Int => Typ::Prelude(PreludeType::Int),
                PreludeType::Float => Typ::Prelude(PreludeType::Float),
                _ => bail!(TypeckError::InvalidUnaryOp {
                    src: self.module.source.clone(),
                    span: location.span.into(),
                    t: inferred_value,
                    op: op.into()
                }),
            },
            // Bool negate / bang `!`
            UnaryOp::Bang => match value_typ {
                PreludeType::Bool => Typ::Prelude(PreludeType::Bool),
                _ => bail!(TypeckError::InvalidUnaryOp {
                    src: self.module.source.clone(),
                    span: location.span.into(),
                    t: inferred_value,
                    op: op.into()
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
                            _ => bail!(TypeckError::ModuleFieldIsPrivate {
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
                            _ => bail!(TypeckError::ModuleFieldIsPrivate {
                                src: self.module.source.clone(),
                                span: field_location.span.into(),
                                name: field_name
                            }),
                        }
                    }
                },
                // Else, raising `module field is not defined`
                None => bail!(TypeckError::ModuleFieldIsNotDefined {
                    src: self.module.source.clone(),
                    span: field_location.span.into(),
                    m: field_module,
                    field: field_name
                }),
            },
            // If module is not defined
            None => bail!(TypeckError::ModuleIsNotDefined { m: field_module }),
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
            None => bail!(TypeckError::EnumVariantIsNotDefined {
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
                            bail!(TypeckError::FieldIsPrivate {
                                src: self.module.source.clone(),
                                span: field_location.span.into(),
                                t: borrowed.name.clone(),
                                field: field_name
                            });
                        }
                    }
                    // Else
                    None => bail!(TypeckError::FieldIsPrivate {
                        src: self.module.source.clone(),
                        span: field_location.span.into(),
                        t: borrowed.name.clone(),
                        field: field_name
                    }),
                },
            },
            None => bail!(TypeckError::FieldIsNotDefined {
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
        container: Expression,
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
                        TypeckWarning::AccessOfDynField {
                            src: self.module.source.clone(),
                            span: field_location.span.into()
                        }
                    );
                    Res::Value(Typ::Dyn)
                }
                // Else
                _ => bail!(TypeckError::CouldNotResolveFieldsIn {
                    src: self.module.source.clone(),
                    span: field_location.span.into(),
                    res: container_inferred
                }),
            },
            // Else
            _ => bail!(TypeckError::CouldNotResolveFieldsIn {
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
        what: Expression,
        args: Vec<Expression>,
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
                    bail!(TypeckError::InvalidArgs {
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
                        bail!(TypeckError::InvalidArgs {
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
                        TypeckWarning::CallOfDyn {
                            src: self.module.source.clone(),
                            span: location.span.into()
                        }
                    );
                    CallResult::FromDyn
                }
                // Else
                _ => bail!(TypeckError::CouldNotCall {
                    src: self.module.source.clone(),
                    span: location.span.into(),
                    res: function
                }),
            },
            // Variant
            Res::Variant(en, variant) => {
                if variant.params.values().cloned().collect::<Vec<Typ>>() != args {
                    bail!(TypeckError::InvalidArgs {
                        src: self.module.source.clone(),
                        params_span: variant.location.span.clone().into(),
                        span: location.span.into()
                    })
                } else {
                    CallResult::FromEnum(Typ::Enum(en.clone()))
                }
            }
            _ => bail!(TypeckError::CouldNotCall {
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
                "unit" => Typ::Unit,
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
                                bail!(TypeckError::TypeIsPrivate {
                                    src: self.module.source.clone(),
                                    span: location.span.into(),
                                    t: t.value.clone()
                                })
                            }
                        }
                        ModDef::Variable(_) => bail!(TypeckError::CouldNotUseValueAsType {
                            src: self.module.source.clone(),
                            span: location.clone().span.into(),
                            v: name
                        }),
                    },
                    None => bail!(TypeckError::TypeIsNotDefined {
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
    pub(crate) fn infer_resolution(&mut self, expr: Expression) -> Res {
        match expr {
            Expression::PrefixVar { location, name } => self.infer_get(location, name),
            Expression::SuffixVar {
                location,
                container,
                name,
            } => self.infer_field_access(location, *container, name),
            Expression::Call {
                location,
                what,
                args,
            } => match self.infer_call(location.clone(), *what, args) {
                CallResult::FromFunction(typ, _) => Res::Value(typ),
                CallResult::FromType(t) => Res::Value(t),
                CallResult::FromEnum(e) => Res::Value(e),
                CallResult::FromDyn => Res::Value(Typ::Dyn),
            },
            expr => bail!(TypeckError::UnexpectedExprInResolution {
                expr: format!("{expr:?}").into()
            }),
        }
    }

    /// Infers anonumous fn
    fn infer_anonymous_fn(
        &mut self,
        location: Address,
        params: Vec<Parameter>,
        body: Block,
        ret_type: Option<TypePath>,
    ) -> Typ {
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
        let block_location = body.location.clone();
        let inferred_block = self.infer_block(body);
        self.solver.solve(Equation::Unify(
            (location, ret),
            (block_location, inferred_block),
        ));
        self.resolver.pop_rib();

        // result
        Typ::Function(RcPtr::new(function))
    }

    /// Analyzes single pattern
    fn analyze_pattern(&mut self, inferred_what: Typ, case: &Case, pat: &Pattern) {
        // matching pattern
        match pat.clone() {
            Pattern::Unwrap { en, fields } => {
                // inferring resolution, and checking
                // that is a enum variant
                let res = self.infer_resolution(en);
                match &res {
                    Res::Variant(en, variant) => {
                        // If types aren't equal
                        let en_typ = Typ::Enum(en.clone());
                        if inferred_what != en_typ {
                            bail!(TypeckError::TypesMissmatch {
                                src: self.module.source.clone(),
                                span: case.address.span.clone().into(),
                                expected: en_typ,
                                got: inferred_what.clone()
                            });
                        }
                        // If types equal, checking fields existence
                        else {
                            fields.into_iter().for_each(|field| {
                                match variant.params.get(&field.1) {
                                    // Defining field with it's type, if it exists
                                    Some(typ) => {
                                        self.resolver.define(
                                            &self.module.source,
                                            &field.0,
                                            &field.1,
                                            Def::Local(typ.clone()),
                                        );
                                    }
                                    None => bail!(TypeckError::EnumVariantFieldIsNotDefined {
                                        src: self.module.source.clone(),
                                        span: case.address.span.clone().into(),
                                        res: res.clone(),
                                        field: field.1
                                    }),
                                }
                            });
                        }
                    }
                    _ => bail!(TypeckError::WrongUnwrapPattern {
                        src: self.module.source.clone(),
                        span: case.address.span.clone().into(),
                        got: res
                    }),
                }
            }
            Pattern::Int(_) => {
                let typ = Typ::Prelude(PreludeType::Int);
                if inferred_what != typ {
                    bail!(TypeckError::TypesMissmatch {
                        src: self.module.source.clone(),
                        span: case.address.span.clone().into(),
                        expected: inferred_what,
                        got: typ
                    })
                }
            }
            Pattern::Float(_) => {
                let typ = Typ::Prelude(PreludeType::Float);
                if inferred_what != typ {
                    bail!(TypeckError::TypesMissmatch {
                        src: self.module.source.clone(),
                        span: case.address.span.clone().into(),
                        expected: inferred_what,
                        got: typ
                    })
                }
            }
            Pattern::String(_) => {
                let typ = Typ::Prelude(PreludeType::String);
                if inferred_what != typ {
                    bail!(TypeckError::TypesMissmatch {
                        src: self.module.source.clone(),
                        span: case.address.span.clone().into(),
                        expected: inferred_what,
                        got: typ
                    })
                }
            }
            Pattern::Bool(_) => {
                let typ = Typ::Prelude(PreludeType::Bool);
                if inferred_what != typ {
                    bail!(TypeckError::TypesMissmatch {
                        src: self.module.source.clone(),
                        span: case.address.span.clone().into(),
                        expected: inferred_what,
                        got: typ
                    })
                }
            }
            Pattern::Wildcard => {}
            Pattern::Variant(var) => {
                // inferring resolution, and checking
                // that is a enum variant
                let res = self.infer_resolution(var);
                match &res {
                    Res::Variant(en, _) => {
                        // If types aren't equal
                        let en_typ = Typ::Enum(en.clone());
                        if inferred_what != en_typ {
                            bail!(TypeckError::TypesMissmatch {
                                src: self.module.source.clone(),
                                span: case.address.span.clone().into(),
                                expected: en_typ,
                                got: inferred_what.clone()
                            });
                        }
                    }
                    _ => bail!(TypeckError::WrongVariantPattern {
                        src: self.module.source.clone(),
                        span: case.address.span.clone().into(),
                        got: res
                    }),
                }
            }
            Pattern::BindTo(name) => {
                self.resolver.define(
                    &self.module.source,
                    &case.address,
                    &name,
                    Def::Local(inferred_what.clone()),
                );
            }
            Pattern::Or(pat1, pat2) => {
                self.analyze_pattern(inferred_what.clone(), case, &pat1);
                self.analyze_pattern(inferred_what, case, &pat2);
            }
        }
    }

    /// Infers pattern matching
    pub(crate) fn infer_pattern_matching(&mut self, what: Expression, cases: Vec<Case>) -> Typ {
        // inferring matchable
        let inferred_what = self.infer_expr(what);
        // to unify
        let mut to_unify = Vec::new();
        // type checking cases
        for case in cases.clone() {
            // pattern scope start
            self.resolver.push_rib(RibKind::Pattern);
            // analyzing pattern
            self.analyze_pattern(inferred_what.clone(), &case, &case.pattern);
            // analyzing body
            let case_location = case.body.location.clone();
            let inferred_case = self.infer_block(case.body);
            to_unify.push((case_location, inferred_case));
            // pattern scope end
            self.resolver.pop_rib();
        }
        // solved type
        let typ = self.solver.solve(Equation::UnifyMany(to_unify));
        let checked = ExMatchCx::check(self, inferred_what, cases);
        // checking all cases covered
        if checked { typ } else { Typ::Unit }
    }

    /// Infers if
    fn infer_if(
        &mut self,
        location: Address,
        logical: Expression,
        body: Block,
        else_branches: Vec<ElseBranch>,
    ) -> Typ {
        // pushing rib
        self.resolver.push_rib(RibKind::Conditional);
        // inferring logical
        let inferred_logical = self.infer_expr(logical);
        match inferred_logical {
            Typ::Prelude(PreludeType::Bool) => {}
            _ => {
                bail!(TypeckError::ExpectedLogicalInIf {
                    src: self.module.source.clone(),
                    span: location.span.into()
                })
            }
        }
        // inferring block
        let if_location = body.location.clone();
        let mut to_unify = vec![(if_location, self.infer_block(body))];
        // popping rib
        self.resolver.pop_rib();
        // else reached
        let mut else_reached = false;
        // analyzing else branches
        for branch in else_branches {
            match branch {
                ElseBranch::Elif { logical, body, .. } => {
                    // inferring logical
                    let logical_location = logical.location();
                    let inferred_logical = self.infer_expr(logical);
                    match inferred_logical {
                        Typ::Prelude(PreludeType::Bool) => {}
                        _ => {
                            bail!(TypeckError::ExpectedLogicalInIf {
                                src: self.module.source.clone(),
                                span: logical_location.span.into()
                            })
                        }
                    }
                    // inferring block
                    let branch_location = body.location.clone();
                    let inferred = self.infer_block(body);
                    to_unify.push((branch_location, inferred));
                }
                ElseBranch::Else { body, .. } => {
                    // inferring block
                    let branch_location = body.location.clone();
                    let inferred = self.infer_block(body);
                    to_unify.push((branch_location, inferred));
                    else_reached = true;
                }
            }
        }
        // checking else reached
        if else_reached {
            self.solver.solve(Equation::UnifyMany(to_unify))
        } else {
            Typ::Unit
        }
    }

    /// Infers expression
    pub(crate) fn infer_expr(&mut self, expr: Expression) -> Typ {
        match expr {
            Expression::Float { .. } => Typ::Prelude(PreludeType::Float),
            Expression::Int { .. } => Typ::Prelude(PreludeType::Int),
            Expression::String { .. } => Typ::Prelude(PreludeType::String),
            Expression::Bool { .. } => Typ::Prelude(PreludeType::Bool),
            Expression::Bin {
                location,
                left,
                right,
                op,
            } => self.infer_binary(location, op, *left, *right),
            Expression::Unary {
                location,
                value,
                op,
            } => self.infer_unary(location, op, *value),
            Expression::PrefixVar { location, name } => self
                .infer_get(location.clone(), name)
                .unwrap_typ(&self.module.source, &location),
            Expression::SuffixVar {
                location,
                container,
                name,
            } => self
                .infer_field_access(location.clone(), *container, name)
                .unwrap_typ(&self.module.source, &location),
            Expression::Call {
                location,
                what,
                args,
            } => match self.infer_call(location.clone(), *what, args) {
                CallResult::FromFunction(typ, _) => typ,
                CallResult::FromType(t) => t,
                CallResult::FromEnum(e) => e,
                CallResult::FromDyn => Typ::Dyn,
            },
            Expression::Function {
                location,
                params,
                body,
                typ,
            } => self.infer_anonymous_fn(location, params, body, typ),
            Expression::Match { value, cases, .. } => self.infer_pattern_matching(*value, cases),
            Expression::If {
                location,
                logical,
                body,
                else_branches,
            } => self.infer_if(location, *logical, body, else_branches),
        }
    }
}
