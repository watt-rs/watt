/// Imports
use crate::analyze::{
    env::{EnvironmentType, EnvironmentsStack},
    errors::AnalyzeError,
    rc_ptr::RcPtr,
};
use ecow::EcoString;
use miette::NamedSource;
use oil_ast::ast::{Publicity, TypePath};
use oil_common::{address::Address, bail};
use oil_ir::ir::{
    IrBinaryOp, IrBlock, IrDeclaration, IrExpression, IrFunction, IrModule, IrParameter,
    IrStatement, IrUnaryOp, IrVariable,
};
use std::{cell::RefCell, collections::HashMap, fmt::Debug};

/// Prelude type
#[derive(Debug, Clone, PartialEq)]
pub enum PreludeType {
    Int,
    Float,
    Bool,
    String,
}

/// Custom type
#[derive(Clone)]
pub struct Type {
    pub location: Address,
    pub name: EcoString,
    pub params: Vec<Typ>,
    pub env: HashMap<EcoString, WithPublicity<Typ>>,
}

/// Debug implementation
impl Debug for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Custom({})", self.name)
    }
}

/// Function
#[derive(Clone)]
pub struct Function {
    pub location: Address,
    pub name: EcoString,
    pub params: Vec<Typ>,
    pub ret: Typ,
}

/// Debug implementation
impl Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Function({})", self.name)
    }
}

/// Module
#[derive(Clone)]
pub struct Module {
    pub source: NamedSource<String>,
    pub name: EcoString,
    pub environment: HashMap<EcoString, WithPublicity<Typ>>,
    pub custom_types: HashMap<EcoString, WithPublicity<RcPtr<RefCell<Type>>>>,
}

/// Debug implementation
impl Debug for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Prelude({})", self.name)
    }
}

/// Typ
#[derive(Clone, PartialEq)]
pub enum Typ {
    Prelude(PreludeType),
    Custom(RcPtr<RefCell<Type>>),
    Function(RcPtr<Function>),
    Void,
}

/// Debug implementation
impl Debug for Typ {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Prelude(prelude) => write!(f, "Type(Prelude({prelude:?}))"),
            Self::Custom(custom) => write!(f, "Type(Custom({}))", custom.borrow().name),
            Self::Function(function) => write!(f, "Type(Function({}))", function.name),
            Self::Void => write!(f, "Void"),
        }
    }
}

/// T with publicity
#[derive(Debug, Clone, PartialEq)]
pub struct WithPublicity<T: Clone + PartialEq> {
    publicity: Publicity,
    value: T,
}

/// Module analyzer
pub struct ModuleAnalyzer<'pkg> {
    module: &'pkg IrModule,
    environments_stack: EnvironmentsStack,
    custom_types: HashMap<EcoString, WithPublicity<RcPtr<RefCell<Type>>>>,
    modules: HashMap<EcoString, &'pkg Module>,
}

/// Implementation
impl<'pkg> ModuleAnalyzer<'pkg> {
    /// Creates new module analyzer
    pub fn new(modules: HashMap<EcoString, &'pkg Module>, module: &'pkg IrModule) -> Self {
        Self {
            module,
            environments_stack: EnvironmentsStack::new(),
            custom_types: HashMap::new(),
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
            | IrBinaryOp::Le
            | IrBinaryOp::Lt
            | IrBinaryOp::Ge
            | IrBinaryOp::Gt
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
            // conditional
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
    fn infer_get(&self, location: Address, name: EcoString) -> Typ {
        self.environments_stack
            .lookup(&self.module.source, &location, &name)
    }

    /// Infers access
    fn infer_field_access(
        &self,
        field_location: Address,
        container: IrExpression,
        field_name: EcoString,
    ) -> Typ {
        match container {
            IrExpression::Get { location, name } => {
                if self.environments_stack.exists(&name) {
                    let container_inferred =
                        self.environments_stack
                            .lookup(&self.module.source, &location, &name);
                    match container_inferred {
                        Typ::Custom(t) => match t.borrow().env.get(&field_name) {
                            Some(field) => {
                                if field.publicity != Publicity::Private || name == "self" {
                                    field.value.clone()
                                } else {
                                    bail!(AnalyzeError::FieldIsPrivate {
                                        src: self.module.source.clone(),
                                        span: field_location.span.into(),
                                        field: field_name
                                    })
                                }
                            }
                            None => bail!(AnalyzeError::FieldIsNotDefined {
                                src: self.module.source.clone(),
                                span: field_location.span.into(),
                                t: t.borrow().name.clone(),
                                field: name
                            }),
                        },
                        _ => bail!(AnalyzeError::InvalidFieldAccess {
                            src: self.module.source.clone(),
                            span: field_location.span.into(),
                            t: container_inferred
                        }),
                    }
                } else if self.modules.contains_key(&name) {
                    let module = self.modules.get(&name).unwrap();
                    match module.environment.get(&field_name) {
                        Some(field) => {
                            if field.publicity != Publicity::Private {
                                field.value.clone()
                            } else {
                                bail!(AnalyzeError::FieldIsPrivate {
                                    src: self.module.source.clone(),
                                    span: field_location.span.into(),
                                    field: field_name
                                })
                            }
                        }
                        None => bail!(AnalyzeError::FieldIsNotDefined {
                            src: module.source.clone(),
                            span: field_location.span.into(),
                            t: module.name.clone(),
                            field: field_name
                        }),
                    }
                } else {
                    bail!(AnalyzeError::VariableIsNotDefined {
                        src: self.module.source.clone(),
                        span: field_location.span.into()
                    })
                }
            }
            _ => {
                let container_inferred = self.infer_expr(container);
                match container_inferred {
                    Typ::Custom(t) => match t.borrow().env.get(&field_name) {
                        Some(field) => field.value.clone(),
                        None => bail!(AnalyzeError::FieldIsNotDefined {
                            src: self.module.source.clone(),
                            span: field_location.span.into(),
                            t: t.borrow().name.clone(),
                            field: field_name
                        }),
                    },
                    _ => bail!(AnalyzeError::InvalidFieldAccess {
                        src: self.module.source.clone(),
                        span: field_location.span.into(),
                        t: container_inferred
                    }),
                }
            }
        }
    }

    /// Infers type path
    fn infer_type_path(&self, path: TypePath) -> Typ {
        match path {
            TypePath::Local { location, name } => match name.as_str() {
                "int" => Typ::Prelude(PreludeType::Int),
                "float" => Typ::Prelude(PreludeType::Float),
                "bool" => Typ::Prelude(PreludeType::Bool),
                "string" => Typ::Prelude(PreludeType::String),
                _ => match self.custom_types.get(&name) {
                    Some(t) => Typ::Custom(t.value.clone()),
                    None => bail!(AnalyzeError::TypeIsNotDefined {
                        src: self.module.source.clone(),
                        span: location.span.into(),
                        t: name
                    }),
                },
            },
            TypePath::Module {
                location,
                module,
                name,
            } => {
                let m = match self.modules.get(&module) {
                    Some(m) => m,
                    None => todo!(),
                };
                let typ = match m.custom_types.get(&name) {
                    Some(t) => {
                        if t.publicity != Publicity::Private {
                            Typ::Custom(t.value.clone())
                        } else {
                            bail!(AnalyzeError::TypeIsPrivate {
                                src: self.module.source.clone(),
                                span: location.span.into(),
                                t: name
                            })
                        }
                    }
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

    /// Infers call
    fn infer_call(
        &self,
        location: Address,
        what: IrExpression,
        args: Vec<IrExpression>,
    ) -> (Typ, RcPtr<Function>) {
        let function = self.infer_expr(what);
        let args = args
            .iter()
            .map(|a| self.infer_expr(a.clone()))
            .collect::<Vec<Typ>>();
        match function {
            Typ::Function(f) => {
                if f.params != args {
                    bail!(AnalyzeError::InvalidArgs {
                        src: self.module.source.clone(),
                        params_span: f.location.span.clone().into(),
                        span: location.span.into()
                    })
                } else {
                    return (f.ret.clone(), f);
                }
            }
            _ => bail!(AnalyzeError::CouldNotCall {
                src: self.module.source.clone(),
                span: location.span.into(),
                t: function
            }),
        }
    }

    /// Infers new
    fn infer_new(&self, location: Address, what: TypePath, args: Vec<IrExpression>) -> Typ {
        let typ = self.infer_type_path(what);
        let args = args
            .iter()
            .map(|a| self.infer_expr(a.clone()))
            .collect::<Vec<Typ>>();
        match typ.clone() {
            Typ::Custom(t) => {
                if t.borrow().params != args {
                    bail!(AnalyzeError::InvalidArgs {
                        src: self.module.source.clone(),
                        params_span: t.borrow().location.span.clone().into(),
                        span: location.span.into()
                    })
                } else {
                    return typ;
                }
            }
            _ => bail!(AnalyzeError::CouldNotInstantiate {
                src: self.module.source.clone(),
                span: location.span.into(),
                t: typ
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
                (Typ::Void, function) => bail!(AnalyzeError::CallExprReturnTypeIsVoid {
                    src: self.module.source.clone(),
                    span: location.span.into(),
                    definition_span: function.location.clone().span.into()
                }),
                (t, _) => t,
            },
            IrExpression::New {
                location,
                what,
                args,
            } => self.infer_new(location, what, args),
            IrExpression::Range { .. } => todo!(),
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
                let annotated = self.infer_type_path(annotated_path);
                if inferred_value != annotated {
                    bail!(AnalyzeError::MissmatchedTypeAnnotation {
                        src: self.module.source.clone(),
                        span: location.span.into(),
                        expected: annotated,
                        got: inferred_value
                    })
                }
            }
            None => {}
        }
        match name.as_str() {
            "self" => bail!(AnalyzeError::SelfVariableDeclared {
                src: self.module.source.clone(),
                span: location.span.into(),
            }),
            _ => self.environments_stack.define(
                &self.module.source,
                &location,
                &name,
                inferred_value,
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

    /// Analyzes funciton
    fn analyze_function(
        &mut self,
        location: Address,
        name: EcoString,
        params: Vec<IrParameter>,
        body: IrBlock,
        ret_type: Option<TypePath>,
    ) {
        // inferring return type
        let ret = ret_type.map_or(Typ::Void, |t| self.infer_type_path(t));
        self.environments_stack
            .push(EnvironmentType::Function(ret.clone()));

        // inferring params
        let params = params
            .into_iter()
            .map(|p| (p.name, self.infer_type_path(p.typ.clone())))
            .collect::<HashMap<EcoString, Typ>>();

        params.iter().for_each(|p| {
            self.environments_stack
                .define(&self.module.source, &location, p.0, p.1.clone())
        });

        // inferring body
        self.analyze_block(body);
        self.environments_stack.pop();

        // creating and defining function
        let function = Function {
            location: location.clone(),
            name: name.clone(),
            params: params.into_iter().map(|(_, v)| v).collect::<Vec<Typ>>(),
            ret,
        };
        self.environments_stack.define(
            &self.module.source.clone(),
            &location,
            &name,
            Typ::Function(RcPtr::new(function)),
        );
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
                // pushing env
                self.environments_stack.push(EnvironmentType::Conditional);
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
                // popping env
                self.environments_stack.pop();
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
                // pushing env
                self.environments_stack.push(EnvironmentType::Loop);
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
                // popping env
                self.environments_stack.pop();
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
            } => self.analyze_function(location, name, params, body, typ),
            IrStatement::Break { location } => {
                if !self.environments_stack.contains_env(EnvironmentType::Loop) {
                    bail!(AnalyzeError::BreakWithoutLoop {
                        src: self.module.source.clone(),
                        span: location.span.into(),
                    })
                }
            }
            IrStatement::Continue { location } => {
                if !self.environments_stack.contains_env(EnvironmentType::Loop) {
                    bail!(AnalyzeError::ContinueWithoutLoop {
                        src: self.module.source.clone(),
                        span: location.span.into(),
                    })
                }
            }
            IrStatement::Return { location, value } => {
                let inferred_value = self.infer_expr(value);
                match self.environments_stack.contains_function() {
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
        let ret = ret_type.map_or(Typ::Void, |t| self.infer_type_path(t));
        self.environments_stack
            .push(EnvironmentType::Function(ret.clone()));

        // inferring params
        let params = params
            .into_iter()
            .map(|p| (p.name, self.infer_type_path(p.typ.clone())))
            .collect::<HashMap<EcoString, Typ>>();

        params.iter().for_each(|p| {
            self.environments_stack
                .define(&self.module.source, &location, p.0, p.1.clone())
        });

        self.environments_stack.define(
            &self.module.source,
            &location,
            &"self".into(),
            Typ::Custom(type_.clone()),
        );

        // inferring body
        self.analyze_block(body);
        self.environments_stack.pop();

        // creating and defining function
        let function = Function {
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
            .map(|p| (p.name, (p.location, self.infer_type_path(p.typ))))
            .collect::<HashMap<EcoString, (Address, Typ)>>();

        // construction type
        let type_ = RcPtr::new(RefCell::new(Type {
            location: location.clone(),
            name: name.clone(),
            params: inferred_params.iter().map(|p| p.1.1.clone()).collect(),
            env: HashMap::new(),
        }));

        // params env start
        self.environments_stack
            .push(EnvironmentType::ConstructorParams);

        // params
        inferred_params.into_iter().for_each(|p| {
            self.environments_stack
                .define(&self.module.source, &p.1.0, &p.0, p.1.1);
        });

        // fields env start
        self.environments_stack.push(EnvironmentType::Fields);

        // fields
        fields.clone().into_iter().for_each(|f| {
            self.analyze_define(f.location, f.name, f.value, f.typ);
        });

        // fields env end
        let analyzed_fields = match self.environments_stack.pop() {
            Some(fields) => fields.1,
            None => bail!(AnalyzeError::EnvironmentsStackIsEmpty),
        };

        // params env end
        self.environments_stack.pop();

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

        // defining type, if not already defined
        match self.custom_types.get(&name) {
            Some(_) => bail!(AnalyzeError::TypeIsAlreadyDefined {
                src: self.module.source.clone(),
                span: location.span.into(),
                t: name
            }),
            None => {
                self.custom_types.insert(
                    name,
                    WithPublicity {
                        publicity,
                        value: type_,
                    },
                );
            }
        }
    }

    /// Analyzes declaration
    pub fn analyze_declaration(&mut self, declaration: IrDeclaration) {
        match declaration {
            IrDeclaration::Function(ir_function) => self.analyze_function(
                ir_function.location,
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
        }
    }

    /// Performs analyze of module
    pub fn analyze(&mut self) {
        self.environments_stack.push(EnvironmentType::Module);
        for definition in self.module.clone().definitions {
            self.analyze_declaration(definition)
        }
    }
}
