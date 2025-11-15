/// Imports
use crate::{
    cx::module::ModuleCx,
    errors::TypeckError,
    ex::ExMatchCx,
    inference::equation::Equation,
    typ::{
        def::{ModuleDef, TypeDef},
        res::Res,
        typ::{Function, Parameter, PreludeType, Typ},
    },
    warnings::TypeckWarning,
};
use ecow::EcoString;
use indexmap::IndexMap;
use std::{collections::HashMap, rc::Rc};
use watt_ast::ast::{
    self, BinaryOp, Block, Case, Either, ElseBranch, Expression, Pattern, Publicity, TypePath,
    UnaryOp,
};
use watt_common::{address::Address, bail, warn};

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
                            op
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
                                op
                            }),
                        }
                    }
                    _ => bail!(TypeckError::InvalidBinaryOp {
                        src: self.module.source.clone(),
                        span: location.span.into(),
                        a: left_typ,
                        b: right_typ,
                        op
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
                    op
                }),
            },
            // Bool negate / bang `!`
            UnaryOp::Bang => match value_typ {
                PreludeType::Bool => Typ::Prelude(PreludeType::Bool),
                _ => bail!(TypeckError::InvalidUnaryOp {
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
        self.resolver.resolve(&location, &name)
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
                    ModuleDef::Type(ty) => {
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
                    ModuleDef::Const(var) => {
                        match var.publicity {
                            // If coonstant is public, we resolved field
                            Publicity::Public => Res::Value(var.value.clone()),
                            // Else, raising `module field is private`
                            _ => bail!(TypeckError::ModuleFieldIsPrivate {
                                src: self.module.source.clone(),
                                span: field_location.span.into(),
                                name: field_name
                            }),
                        }
                    }
                    ModuleDef::Function(f) => {
                        match f.publicity {
                            // If coonstant is public, we resolved field
                            Publicity::Public => Res::Value(Typ::Function(f.value.clone())),
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
        &mut self,
        ty: Typ,
        name: EcoString,
        field_location: Address,
        field_name: EcoString,
    ) -> Res {
        // Finding field
        match ty
            .variants(&mut self.solver.hydrator)
            .iter()
            .find(|f| f.name == field_name)
        {
            Some(f) => Res::Variant(ty, f.clone()),
            None => bail!(TypeckError::FieldIsNotDefined {
                src: self.module.source.clone(),
                span: field_location.span.into(),
                t: name,
                field: field_name
            }),
        }
    }

    /// Infers struct field access
    fn infer_struct_field_access(
        &mut self,
        ty: Typ,
        name: EcoString,
        field_location: Address,
        field_name: EcoString,
    ) -> Res {
        // Finding field
        match ty
            .fields(&mut self.solver.hydrator)
            .iter()
            .find(|f| f.name == field_name)
        {
            Some(f) => Res::Value(f.typ.clone()),
            None => bail!(TypeckError::FieldIsNotDefined {
                src: self.module.source.clone(),
                span: field_location.span.into(),
                t: name,
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
            Res::Custom(TypeDef::Enum(en)) => {
                let instantiated = Typ::Enum(
                    en.clone(),
                    self.solver
                        .hydrator
                        .mk_generics(&en.borrow().generics, &mut HashMap::new()),
                );
                self.infer_enum_field_access(
                    instantiated,
                    en.borrow().name.clone(),
                    field_location,
                    field_name,
                )
            }
            // Type field access
            Res::Value(typ) => match typ {
                // Custom Type
                it @ Typ::Struct(ty, _) => {
                    let res = self.infer_struct_field_access(
                        it.clone(),
                        ty.borrow().name.clone(),
                        field_location,
                        field_name,
                    );
                    res
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
    ) -> Res {
        let function = self.infer_resolution(what);
        let args = args
            .iter()
            .map(|a| (a.location(), self.infer_expr(a.clone())))
            .collect::<Vec<(Address, Typ)>>();

        match function.clone() {
            // Custom type
            Res::Custom(TypeDef::Struct(ty)) => {
                let instantiated = Typ::Struct(
                    ty.clone(),
                    self.solver
                        .hydrator
                        .mk_generics(&ty.borrow().generics, &mut HashMap::new()),
                );

                instantiated
                    .fields(&mut self.solver.hydrator)
                    .into_iter()
                    .zip(args)
                    .for_each(|(p, a)| {
                        self.solver.solve(Equation::Unify((p.location, p.typ), a));
                    });

                Res::Value(instantiated)
            }
            // Value
            Res::Value(t) => match t {
                // Function
                Typ::Function(f) => {
                    let f = self.solver.hydrator.mk_function(f, &mut HashMap::new());
                    f.params.iter().cloned().zip(args).for_each(|(p, a)| {
                        self.solver.unify(p.location, p.typ, a.0, a.1);
                    });
                    Res::Value(Typ::Function(f))
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
                variant.fields.iter().cloned().zip(args).for_each(|(p, a)| {
                    self.solver.solve(Equation::Unify((p.location, p.typ), a));
                });

                Res::Value(en)
            }
            _ => bail!(TypeckError::CouldNotCall {
                src: self.module.source.clone(),
                span: location.span.into(),
                res: function
            }),
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
            } => self.infer_call(location.clone(), *what, args),
            expr => bail!(TypeckError::UnexpectedExprInResolution {
                expr: format!("{expr:?}").into()
            }),
        }
    }

    /// Infers anonumous fn
    fn infer_anonymous_fn(
        &mut self,
        location: Address,
        params: Vec<ast::Parameter>,
        body: Either<Block, Box<Expression>>,
        ret_type: Option<TypePath>,
    ) -> Typ {
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

        // creating function
        let function = Function {
            location: location.clone(),
            name: EcoString::from("$anonymous"),
            generics: Vec::new(),
            params: params.clone().into_values().collect(),
            ret: ret.clone(),
        };

        // pushing new scope
        self.resolver.push_rib();

        // defining params in new scope
        params
            .into_iter()
            .for_each(|p| self.resolver.define_local(&location, &p.0, p.1.typ, false));

        // inferring body
        let (block_location, inferred_block) = match body {
            Either::Left(block) => (block.location.clone(), self.infer_block(block)),
            Either::Right(expr) => (expr.location(), self.infer_expr(*expr)),
        };
        self.solver.solve(Equation::Unify(
            (location, ret),
            (block_location, inferred_block),
        ));
        self.resolver.pop_rib();

        // result
        Typ::Function(Rc::new(function))
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
                        if inferred_what != *en {
                            bail!(TypeckError::TypesMissmatch {
                                src: self.module.source.clone(),
                                span: case.address.span.clone().into(),
                                expected: en.clone(),
                                got: inferred_what.clone()
                            });
                        }
                        // If types equal, checking fields existence
                        else {
                            fields.into_iter().for_each(|field| {
                                if !variant.fields.iter().any(|f| f.name == field.1) {
                                    bail!(TypeckError::EnumVariantFieldIsNotDefined {
                                        src: self.module.source.clone(),
                                        span: field.0.span.into(),
                                        res: res.clone(),
                                        field: field.1
                                    })
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
                        if inferred_what != *en {
                            bail!(TypeckError::TypesMissmatch {
                                src: self.module.source.clone(),
                                span: case.address.span.clone().into(),
                                expected: en.clone(),
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
                self.resolver
                    .define_local(&case.address, &name, inferred_what.clone(), false);
            }
            Pattern::Or(pat1, pat2) => {
                self.analyze_pattern(inferred_what.clone(), case, &pat1);
                self.analyze_pattern(inferred_what, case, &pat2);
            }
        }
    }

    /// Infers pattern matching
    pub(crate) fn infer_pattern_matching(
        &mut self,
        location: Address,
        what: Expression,
        cases: Vec<Case>,
    ) -> Typ {
        // inferring matchable
        let inferred_what = self.infer_expr(what);
        // to unify
        let mut to_unify = Vec::new();
        // type checking cases
        for case in cases.clone() {
            // pattern scope start
            self.resolver.push_rib();
            // analyzing pattern
            self.analyze_pattern(inferred_what.clone(), &case, &case.pattern);
            // analyzing body
            let (case_location, inferred_case) = match case.body {
                Either::Left(block) => (block.location.clone(), self.infer_block(block)),
                Either::Right(expr) => (expr.location(), self.infer_expr(expr)),
            };
            to_unify.push((case_location, inferred_case));
            // pattern scope end
            self.resolver.pop_rib();
        }
        // solved type
        let typ = self.solver.solve(Equation::UnifyMany(to_unify));
        let checked = ExMatchCx::check(self, inferred_what, cases);
        // checking all cases covered
        if checked {
            typ
        } else {
            warn!(
                self.package,
                TypeckWarning::NonExhaustive {
                    src: location.source,
                    span: location.span.into()
                }
            );
            Typ::Unit
        }
    }

    /// Infers if
    fn infer_if(
        &mut self,
        location: Address,
        logical: Expression,
        body: Either<Block, Box<Expression>>,
        else_branches: Vec<ElseBranch>,
    ) -> Typ {
        // pushing rib
        self.resolver.push_rib();
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
        let (if_location, inferred_if) = match body {
            Either::Left(block) => (block.location.clone(), self.infer_block(block)),
            Either::Right(expr) => (expr.location(), self.infer_expr(*expr)),
        };
        let mut to_unify = vec![(if_location, inferred_if)];
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
                    let (branch_location, inferred_branch) = match body {
                        Either::Left(block) => (block.location.clone(), self.infer_block(block)),
                        Either::Right(expr) => (expr.location(), self.infer_expr(expr)),
                    };
                    to_unify.push((branch_location, inferred_branch));
                }
                ElseBranch::Else { body, .. } => {
                    // inferring block
                    let (branch_location, inferred_branch) = match body {
                        Either::Left(block) => (block.location.clone(), self.infer_block(block)),
                        Either::Right(expr) => (expr.location(), self.infer_expr(expr)),
                    };
                    to_unify.push((branch_location, inferred_branch));
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
        // Inferencing expression
        let result = match expr {
            Expression::Float { .. } => Typ::Prelude(PreludeType::Float),
            Expression::Int { .. } => Typ::Prelude(PreludeType::Int),
            Expression::String { .. } => Typ::Prelude(PreludeType::String),
            Expression::Bool { .. } => Typ::Prelude(PreludeType::Bool),
            Expression::Todo { location } => {
                warn!(
                    self.package,
                    TypeckWarning::FoundTodo {
                        src: self.module.source.clone(),
                        span: location.span.into()
                    }
                );
                Typ::Unit
            }
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
            Expression::PrefixVar { location, name } => {
                self.infer_get(location.clone(), name).unwrap_typ(&location)
            }
            Expression::SuffixVar {
                location,
                container,
                name,
            } => self
                .infer_field_access(location.clone(), *container, name)
                .unwrap_typ(&location),
            Expression::Call {
                location,
                what,
                args,
            } => self
                .infer_call(location.clone(), *what, args)
                .unwrap_typ(&location),
            Expression::Function {
                location,
                params,
                body,
                typ,
            } => self.infer_anonymous_fn(location, params, body, typ),
            Expression::Match {
                location,
                value,
                cases,
                ..
            } => self.infer_pattern_matching(location, *value, cases),
            Expression::If {
                location,
                logical,
                body,
                else_branches,
            } => self.infer_if(location, *logical, body, else_branches),
        };
        // Applying substs
        self.solver.hydrator.apply(result)
    }
}
