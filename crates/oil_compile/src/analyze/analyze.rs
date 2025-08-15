/// Imports
use crate::{
    analyze::{env::Environment, errors::AnalyzeError},
    project::ProjectCompiler,
};
use ecow::EcoString;
use miette::NamedSource;
use oil_ast::ast::TypePath;
use oil_common::{address::Address, bail};
use oil_ir::ir::{IrBinaryOp, IrDeclaration, IrExpression, IrModule, IrStatement, IrUnaryOp};
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
    pub env: Environment,
}

/// Function
#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub location: Address,
    pub name: EcoString,
    pub params: Vec<Typ>,
    pub env: Environment,
    pub ret: Typ,
}

/// Module
#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub source: NamedSource<String>,
    pub name: EcoString,
    pub environment: Environment,
    pub custom_types: HashMap<EcoString, Rc<Type>>,
}

/// Typ
#[derive(Debug, Clone, PartialEq)]
pub enum Typ {
    Prelude(PreludeType),
    Custom(Rc<Type>),
    Instance(Rc<Type>),
    Function(Rc<Function>),
}

/// Module analyzer
pub struct ModuleAnalyzer<'pkg> {
    module: &'pkg IrModule,
    environments_stack: Vec<Environment>,
    custom_types: HashMap<EcoString, Rc<Type>>,
    modules: HashMap<EcoString, &'pkg Module>,
}

/// Implementation
impl<'pkg> ModuleAnalyzer<'pkg> {
    /// Creates new module analyzer
    pub fn new(modules: HashMap<EcoString, &'pkg Module>, module: &'pkg IrModule) -> Self {
        Self {
            module,
            environments_stack: Vec::new(),
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
        match self.environments_stack.last() {
            Some(env) => env.lookup(&self.module.source, &location, name),
            None => bail!(AnalyzeError::EnvironmentsStackIsEmpty),
        }
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
            Typ::Custom(t) => t.env.lookup(&self.module.source, &location, name),
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
        args_location: Address,
        what: IrExpression,
        args: Vec<IrExpression>,
    ) -> Typ {
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
                        span: args_location.span.into()
                    })
                } else {
                    return f.ret.clone();
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
            IrExpression::Get {
                location,
                base,
                name,
            } => self.infer_get(location, base.map(|base| *base), name),
            IrExpression::Call {
                location,
                args_location,
                base,
                name,
                args,
            } => self.infer_call(location, args_location, base.map(|base| *base), name, args),
            IrExpression::Range { location, from, to } => todo!(),
        }
    }

    /// Analyzes statement
    pub fn analyze_statement(&mut self, statement: IrStatement) {
        todo!()
    }

    /// Analyzes declaration
    pub fn analyze_declaration(&mut self, declaration: IrDeclaration) {
        todo!()
    }

    /// Performs analyze of module
    pub fn analyze(&mut self) {
        for definition in self.module.clone().definitions {
            self.analyze_declaration(definition)
        }
    }
}
