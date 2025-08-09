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
        base: Option<Box<UntypedExpression>>,
        name: UntypedExpression,
    },
    Call {
        base: Option<Box<UntypedExpression>>,
        name: String,
        args: Vec<UntypedExpression>,
    },
    Fn {
        name: String,
        params: Vec<Parameter>,
        body: Box<UntypedBlock>,
        typ: Option<TypePath>,
    },
}

/// Untyped expression
pub enum UntypedExpression {}

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
    dependencies: Vec<Dependency>,
}
