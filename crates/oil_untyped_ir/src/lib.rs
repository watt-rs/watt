/// Imports
use oil_ast::ast::{DependencyPath, Parameter, Publicity, TypePath};
use oil_common::address::Address;
use std::path::PathBuf;

/// Untyped statement
pub enum UntypedStatement {
    If {
        location: Address,
        logical: Box<UntypedExpression>,
        body: Box<UntypedStatement>,
        elseif: Option<Box<UntypedStatement>>,
    },
    While {
        location: Address,
        logical: Box<UntypedExpression>,
        body: Box<UntypedStatement>,
    },
    Define {
        publicity: Publicity,
        location: Address,
        name: String,
        value: Box<UntypedExpression>,
        typ: Option<TypePath>,
    },
    Assign {
        base: Option<Box<UntypedExpression>>,
        name: String,
        value: Box<UntypedExpression>,
    },
    Get {
        location: Address,
        base: Option<Box<UntypedExpression>>,
        name: UntypedExpression,
    },
    Call {
        location: Address,
        base: Option<Box<UntypedExpression>>,
        name: String,
        args: Vec<UntypedExpression>,
    },
    Fn {
        location: Address,
        name: String,
        params: Vec<Parameter>,
        body: Box<UntypedBlock>,
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
        iterable: Box<UntypedExpression>,
        variable: String,
        body: Box<UntypedBlock>,
    },
    Return {
        location: Address,
        value: Box<UntypedExpression>,
    },
}

/// Untyped expression
pub enum UntypedExpression {
    Number {
        location: Address,
        value: String,
    },
    String {
        location: Address,
        value: String,
    },
    Bool {
        location: Address,
        value: String,
    },
    Bin {
        location: Address,
        left: Box<UntypedExpression>,
        right: Box<UntypedExpression>,
        op: BinaryOperator,
    },
    Unary {
        location: Address,
        value: Box<UntypedExpression>,
        op: UnaryOperator,
    },
    Get {
        location: Address,
        previous: Option<Box<UntypedExpression>>,
        name: String,
    },
    Call {
        location: Address,
        previous: Option<Box<UntypedExpression>>,
        name: String,
        args: Vec<UntypedExpression>,
    },
    Range {
        location: Address,
        from: Box<UntypedExpression>,
        to: Box<UntypedExpression>,
    },
}

/// Binary operator
pub enum BinaryOperator {
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
}

/// Unary operator
pub enum UnaryOperator {
    Negate,
    Bang,
}

/// Untyped block
pub struct UntypedBlock {
    nodes: Vec<UntypedStatement>,
}

/// Untyped definition
pub enum UntypedDefinition {
    UntypedFunction {
        name: String,
        publicity: Publicity,
        parameters: Vec<Parameter>,
        body: Box<UntypedBlock>,
        typ: Option<DependencyPath>,
    },
    UntypedType {
        name: String,
        publicity: Publicity,
        constructor: Vec<Parameter>,
        fields: Vec<UntypedBlock>,
        functions: Vec<UntypedBlock>,
    },
    UntypedVariable {
        name: String,
        publicity: Publicity,
        typ: Option<DependencyPath>,
        value: UntypedExpression,
    },
}

/// Untyped module
pub struct UntypedModule {
    name: String,
    path: PathBuf,
    definitions: Vec<UntypedDefinition>,
}
