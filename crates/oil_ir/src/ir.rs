/// Imports
use ecow::EcoString;
use miette::NamedSource;
use oil_ast::ast::{Publicity, TypePath};
use oil_common::address::Address;
use std::sync::Arc;

/// Ir parameter
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct IrParameter {
    pub location: Address,
    pub name: EcoString,
    pub typ: TypePath,
}

/// Ir enum constructor
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct IrEnumConstructor {
    pub location: Address,
    pub name: EcoString,
    pub params: Vec<IrParameter>,
}

/// Ir statement
#[derive(Debug, Clone, PartialEq)]
pub enum IrStatement {
    If {
        location: Address,
        logical: IrExpression,
        body: IrBlock,
        elseif: Option<Box<IrStatement>>,
    },
    While {
        location: Address,
        logical: IrExpression,
        body: IrBlock,
    },
    Define {
        location: Address,
        name: EcoString,
        value: IrExpression,
        typ: Option<TypePath>,
    },
    Assign {
        location: Address,
        what: Box<IrExpression>,
        value: Box<IrExpression>,
    },
    Call {
        location: Address,
        what: Box<IrExpression>,
        args: Vec<IrExpression>,
    },
    Fn {
        location: Address,
        name: EcoString,
        params: Vec<IrParameter>,
        body: IrBlock,
        typ: Option<TypePath>,
    },
    Break {
        location: Address,
    },
    Continue {
        location: Address,
    },
    For {
        location: Address,
        iterable: IrExpression,
        variable: EcoString,
        body: IrBlock,
    },
    Return {
        location: Address,
        value: IrExpression,
    },
}

/// Ir Expression
#[derive(Debug, Clone, PartialEq)]
pub enum IrExpression {
    Float {
        location: Address,
        value: f64,
    },
    Int {
        location: Address,
        value: i64,
    },
    String {
        location: Address,
        value: EcoString,
    },
    Bool {
        location: Address,
        value: EcoString,
    },
    Bin {
        location: Address,
        left: Box<IrExpression>,
        right: Box<IrExpression>,
        op: IrBinaryOp,
    },
    Unary {
        location: Address,
        value: Box<IrExpression>,
        op: IrUnaryOp,
    },
    Get {
        location: Address,
        name: EcoString,
    },
    FieldAccess {
        location: Address,
        container: Box<IrExpression>,
        name: EcoString,
    },
    Call {
        location: Address,
        what: Box<IrExpression>,
        args: Vec<IrExpression>,
    },
    Range {
        location: Address,
        from: Box<IrExpression>,
        to: Box<IrExpression>,
    },
}

/// Binary operator
#[derive(Debug, Clone, PartialEq)]
pub enum IrBinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Or,
    And,
    Xor,
    BitwiseAnd,
    BitwiseOr,
    Mod,
    Eq,
    Neq,
    Gt,
    Lt,
    Ge,
    Le,
}

/// Unary operator
#[derive(Debug, Clone, PartialEq)]
pub enum IrUnaryOp {
    Negate,
    Bang,
}

/// Ir block
#[derive(Debug, Clone, PartialEq)]
pub struct IrBlock {
    pub nodes: Vec<IrStatement>,
}

/// Ir function
#[derive(Debug, Clone, PartialEq)]
pub struct IrFunction {
    pub location: Address,
    pub name: EcoString,
    pub publicity: Publicity,
    pub params: Vec<IrParameter>,
    pub body: IrBlock,
    pub typ: Option<TypePath>,
}

/// Ir variable
#[derive(Debug, Clone, PartialEq)]
pub struct IrVariable {
    pub location: Address,
    pub name: EcoString,
    pub publicity: Publicity,
    pub typ: Option<TypePath>,
    pub value: IrExpression,
}

/// Ir type
#[derive(Debug, Clone, PartialEq)]
pub struct IrType {
    pub location: Address,
    pub name: EcoString,
    pub publicity: Publicity,
    pub constructor: Vec<IrParameter>,
    pub fields: Vec<IrVariable>,
    pub functions: Vec<IrFunction>,
}

/// Ir enum
#[derive(Debug, Clone, PartialEq)]
pub struct IrEnum {
    pub location: Address,
    pub name: EcoString,
    pub publicity: Publicity,
    pub variants: Vec<IrEnumConstructor>,
}

/// Ir declaration
#[derive(Debug, Clone, PartialEq)]
pub enum IrDeclaration {
    Function(IrFunction),
    Variable(IrVariable),
    Type(IrType),
    Enum(IrEnum),
}

/// Ir dependency
#[derive(Debug, Clone, PartialEq)]
pub struct IrDependency {
    pub location: Address,
    pub name: Option<EcoString>,
    pub path: EcoString,
}

/// Ir module
#[derive(Debug, Clone, PartialEq)]
pub struct IrModule {
    pub definitions: Vec<IrDeclaration>,
    pub dependencies: Vec<IrDependency>,
    pub source: NamedSource<Arc<String>>,
}
