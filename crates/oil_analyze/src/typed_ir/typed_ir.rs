/// Imports
use crate::{
    analyse::analyse::Typ,
    untyped_ir::untyped_ir::{BinaryOperator, UnaryOperator},
};
use ecow::EcoString;
use oil_ast::ast::Publicity;
use oil_common::address::Address;

/// Typed parameter
#[derive(Debug)]
pub struct TypedParameter {
    pub name: EcoString,
    pub typ: Typ,
}

/// Typed statement
#[derive(Debug)]
pub enum TypedStatement {
    If {
        location: Address,
        logical: TypedExpression,
        body: TypedBlock,
        elseif: Option<Box<TypedStatement>>,
    },
    While {
        location: Address,
        logical: TypedExpression,
        body: TypedBlock,
    },
    Define {
        location: Address,
        name: EcoString,
        value: TypedExpression,
        typ: Typ,
    },
    Assign {
        location: Address,
        base: Option<TypedExpression>,
        name: EcoString,
        value: TypedExpression,
    },
    Get {
        location: Address,
        base: Option<TypedExpression>,
        name: TypedExpression,
    },
    Call {
        location: Address,
        base: Option<TypedExpression>,
        name: EcoString,
        args: Vec<TypedExpression>,
    },
    Fn {
        location: Address,
        name: EcoString,
        params: Vec<TypedParameter>,
        body: TypedBlock,
        typ: Typ,
    },
    Break {
        location: Address,
    },
    Continue {
        location: Address,
    },
    For {
        location: Address,
        iterable: TypedExpression,
        variable: EcoString,
        body: TypedBlock,
    },
    Return {
        location: Address,
        value: TypedExpression,
    },
}

/// Typed literal
#[derive(Debug)]
pub enum TypedLiteral {
    Float(f64),
    Int(i64),
    String(EcoString),
    Bool(bool),
}

/// Untyped expression
#[derive(Debug)]
pub enum TypedExpression {
    Literal {
        literal: TypedLiteral,
        result: Typ,
    },
    Bin {
        location: Address,
        left: Box<TypedExpression>,
        right: Box<TypedExpression>,
        op: BinaryOperator,
        result: Typ,
    },
    Unary {
        location: Address,
        value: Box<TypedExpression>,
        op: UnaryOperator,
        result: Typ,
    },
    Get {
        location: Address,
        base: Option<Box<TypedExpression>>,
        name: EcoString,
        result: Typ,
    },
    Call {
        location: Address,
        base: Option<Box<TypedExpression>>,
        name: EcoString,
        args: Vec<TypedExpression>,
        result: Typ,
    },
    Range {
        location: Address,
        from: Box<TypedExpression>,
        to: Box<TypedExpression>,
        result: Typ,
    },
}

/// Typed block
#[derive(Debug)]
pub struct TypedBlock {
    pub nodes: Vec<TypedStatement>,
}

/// Typed function
#[derive(Debug)]
pub struct TypedFunction {
    pub location: Address,
    pub name: EcoString,
    pub publicity: Publicity,
    pub params: Vec<TypedParameter>,
    pub body: TypedBlock,
    pub typ: Typ,
}

/// Typed variable
#[derive(Debug)]
pub struct TypedVariable {
    pub location: Address,
    pub name: EcoString,
    pub publicity: Publicity,
    pub typ: Typ,
    pub value: TypedExpression,
}

/// Typed type
#[derive(Debug)]
pub struct TypedType {
    pub location: Address,
    pub name: EcoString,
    pub publicity: Publicity,
    pub constructor: Vec<TypedParameter>,
    pub fields: Vec<TypedVariable>,
    pub functions: Vec<TypedFunction>,
}

/// Typed declaration
#[derive(Debug)]
pub enum TypedDeclaration {
    Function(TypedFunction),
    Variable(TypedVariable),
    Type(TypedType),
}

/// Typed module
#[derive(Debug)]
pub struct TypedModule {
    pub definitions: Vec<TypedDeclaration>,
}
