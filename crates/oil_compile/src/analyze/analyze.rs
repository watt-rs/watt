/// Imports
use crate::{
    analyze::{env::Environment, errors::AnalyzeError},
    project::ProjectCompiler,
};
use ecow::EcoString;
use oil_common::{address::Address, bail};
use oil_ir::ir::{
    IrBinaryOp, IrDeclaration, IrExpression, IrFunction, IrModule, IrParameter, IrStatement,
    IrUnaryOp,
};

/// Prelude types
#[derive(Debug, Clone)]
pub enum PreludeType {
    Int,
    Float,
    Bool,
    String,
}

/// Custom type
#[derive(Debug, Clone)]
pub struct Type {
    pub name: EcoString,
    pub params: Vec<IrParameter>,
    pub env: Environment,
}

/// Typ
#[derive(Debug, Clone)]
pub enum Typ {
    Prelude(PreludeType),
    CustomType(Type),
    Function(IrFunction),
}

/// Module analyzer
pub struct ModuleAnalyzer<'pkg> {
    project_compiler: &'pkg mut ProjectCompiler,
    environments_stack: Vec<Environment>,
    module: &'pkg IrModule,
}

/// Implementation
impl<'pkg> ModuleAnalyzer<'pkg> {
    /// Creates new module analyzer
    pub fn new(project_compiler: &'pkg mut ProjectCompiler, module: &'pkg IrModule) -> Self {
        Self {
            project_compiler,
            environments_stack: Vec::new(),
            module,
        }
    }

    /// Infers binary
    fn infer_binary(
        &mut self,
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
    fn infer_unary(&mut self, location: Address, op: IrUnaryOp, value: IrExpression) -> Typ {
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

    /// Infers expression
    fn infer_expr(&mut self, expr: IrExpression) -> Typ {
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
            } => self.infer_get(location, base, op, *value),
            IrExpression::Call {
                location,
                base,
                name,
                args,
            } => self.infer_call_expr(location, base, op, *value),
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
