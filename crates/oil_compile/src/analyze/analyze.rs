/// Imports
use crate::analyze::{env::EnvironmentsStack, errors::AnalyzeError};
use ecow::EcoString;
use miette::NamedSource;
use oil_ast::ast::{Publicity, TypePath};
use oil_common::{address::Address, bail};
use oil_ir::ir::{
    IrBinaryOp, IrBlock, IrDeclaration, IrExpression, IrModule, IrParameter, IrStatement, IrUnaryOp,
};
use std::{collections::HashMap, rc::Rc};

/// Prelude type
#[derive(Debug, Clone, PartialEq)]
pub enum PreludeType {
    Int,
    Float,
    Bool,
    String,
}

/// Custom type
#[derive(Debug, Clone, PartialEq)]
pub struct Type {
    pub location: Address,
    pub name: EcoString,
    pub params: Vec<Typ>,
    pub env: HashMap<EcoString, WithPublicity<Typ>>,
}

/// Function
#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub location: Address,
    pub name: EcoString,
    pub params: Vec<Typ>,
    pub env: HashMap<EcoString, Typ>,
    pub ret: Typ,
}

/// Module
#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub source: NamedSource<String>,
    pub name: EcoString,
    pub environment: HashMap<EcoString, Typ>,
    pub custom_types: HashMap<EcoString, WithPublicity<Rc<Type>>>,
}

/// Typ
#[derive(Debug, Clone, PartialEq)]
pub enum Typ {
    Prelude(PreludeType),
    Custom(Rc<Type>),
    Instance(Rc<Type>),
    Function(Rc<Function>),
    Void,
}

/// T with publicity
#[derive(Debug, Clone, PartialEq)]
pub struct WithPublicity<T: Clone + PartialEq> {
    publicity: Publicity,
    value: T
}

/// Module analyzer
pub struct ModuleAnalyzer<'pkg> {
    module: &'pkg IrModule,
    environments_stack: EnvironmentsStack,
    custom_types: HashMap<EcoString, WithPublicity<Rc<Type>>>,
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
            IrBinaryOp::Ge
            | IrBinaryOp::Gt
            | IrBinaryOp::Lt
            | IrBinaryOp::Neq
            | IrBinaryOp::Xor
            | IrBinaryOp::And
            | IrBinaryOp::Eq
            | IrBinaryOp::Le
            | IrBinaryOp::Or => {
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
            .lookup(&self.module.source, &location, name)
    }

    /// Infers field access
    fn infer_field_access(
        &self,
        location: Address,
        container: IrExpression,
        name: EcoString,
    ) -> Typ {
        let container_inferred = self.infer_expr(container);
        match container_inferred {
            Typ::Custom(t) => match t.env.get(&name) {
                Some(field) => field.clone(),
                None => bail!(AnalyzeError::FieldIsNotDefined {
                    src: self.module.source.clone(),
                    span: location.span.into(),
                    t: t.name.clone(),
                    field: name
                }),
            },
            _ => bail!(AnalyzeError::InvalidFieldAccess {
                src: self.module.source.clone(),
                span: location.span.into(),
                t: container_inferred
            }),
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
                    Some(t) => Typ::Custom(t.clone()),
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
                    Some(t) => Typ::Custom(t.clone()),
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
    ) -> (Typ, Rc<Function>) {
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
                if inferred_value == annotated {
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
        self.environments_stack
            .define(&self.module.source, &location, name, inferred_value);
    }

    /// Analyzes assignment
    fn analyze_assignment(&mut self, location: Address, what: IrExpression, value: IrExpression) {
        let inferred_value = self.infer_expr(value);
        match what {
            IrExpression::Get { name, .. } => {
                let typ =
                    self.environments_stack
                        .lookup(&self.module.source, &location, name.clone());
                if inferred_value != typ {
                    bail!(AnalyzeError::TypesMissmatch {
                        src: self.module.source.clone(),
                        span: location.span.into(),
                        expected: typ,
                        got: inferred_value
                    })
                }
            }
            IrExpression::FieldAccess {
                container, name, ..
            } => match *container {
                IrExpression::Get { name, .. } => {
                    let module = self.modules.get(&name).map(|m| *m);
                    let variable = self.environments_stack.try_lookup(name);
                    match (module, variable) {
                        (Some(module), Some(variable)) => {
                            if variable != inferred_value {
                                match module.environment.get()
                            }
                        },
                        (None, Some(typ)) => {
                            if inferred_value != typ {
                                bail!(AnalyzeError::TypesMissmatch {
                                    src: self.module.source.clone(),
                                    span: location.span.into(),
                                    expected: typ,
                                    got: inferred_value
                                })
                            }
                        }
                    }
                }
                expr => {
                    let container_inferred = self.infer_expr(expr);
                    match container_inferred {
                        Typ::Instance(instance) => {
                            let typ =
                                instance
                                    .env
                                    .lookup(&self.module.source, &location, name.clone());
                            if inferred_value != typ {
                                bail!(AnalyzeError::TypesMissmatch {
                                    src: self.module.source.clone(),
                                    span: location.span.into(),
                                    expected: typ,
                                    got: inferred_value
                                })
                            }
                        }
                        _ => bail!(AnalyzeError::InvalidFieldAccess {
                            src: self.module.source.clone(),
                            span: location.span.into(),
                            t: container_inferred
                        }),
                    }
                }
            },
            _ => bail!(AnalyzeError::InvalidAssignmentVariable),
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
        self.environments_stack.push(Environment::new());

        // inferring params
        let params = params
            .into_iter()
            .map(|p| (p.name, self.infer_type_path(p.typ.clone())))
            .collect::<HashMap<EcoString, Typ>>();

        params
            .iter()
            .for_each(|p| match self.environments_stack.last_mut() {
                Some(env) => env.define(&self.module.source, &location, p.0.clone(), p.1.clone()),
                None => bail!(AnalyzeError::EnvironmentsStackIsEmpty),
            });

        // inferring body
        self.analyze_block(body);
        let env = match self.environments_stack.pop() {
            Some(env) => env,
            None => bail!(AnalyzeError::EnvironmentsStackIsEmpty),
        };

        // creating and defining function
        let function = Function {
            location: location.clone(),
            name: name.clone(),
            params: params.into_iter().map(|(_, v)| v).collect::<Vec<Typ>>(),
            env,
            ret: ret_type.map_or(Typ::Void, |t| self.infer_type_path(t)),
        };
        match self.environments_stack.pop() {
            Some(mut env) => env.define(
                &self.module.source,
                &location,
                name,
                Typ::Function(Rc::new(function)),
            ),
            None => bail!(AnalyzeError::EnvironmentsStackIsEmpty),
        };
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
                self.environments_stack.push(Environment::new());
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
                self.environments_stack.push(Environment::new());
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
            IrStatement::Break { location } => todo!(),
            IrStatement::Continue { location } => todo!(),
            IrStatement::For {
                location,
                iterable,
                variable,
                body,
            } => todo!(),
            IrStatement::Return { location, value } => todo!(),
        }
    }

    /// Analyzes declaration
    pub fn analyze_declaration(&mut self, declaration: IrDeclaration) {
        todo!()
    }

    /// Performs analyze of module
    pub fn analyze(&mut self) {
        self.environments_stack.push(Environment::new());
        for definition in self.module.clone().definitions {
            self.analyze_declaration(definition)
        }
    }
}
