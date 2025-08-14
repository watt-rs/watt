/// Imports
use ecow::EcoString;
use oil_common::address::Address;
use oil_lex::tokens::Token;

/// Dependency path
/// TODO!@!!!!#!#!!!
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DependencyPath {
    pub address: Address,
    pub module: EcoString,
}

/// Type path
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TypePath {
    Local(EcoString),
    Module { module: EcoString, name: EcoString },
}

/// Parameter
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Parameter {
    pub name: Token,
    pub typ: TypePath,
}

/// Parameter implementation
impl Parameter {
    /// Creates new parameter
    pub fn new(name: Token, typ: TypePath) -> Self {
        Self { name, typ }
    }
}

/// Publicity
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Publicity {
    Public,
    Private,
    None,
}

/// Ast node
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[allow(dead_code)]
pub enum Node {
    Block {
        body: Vec<Node>,
    },
    Number {
        value: Token,
    },
    String {
        value: Token,
    },
    Bool {
        value: Token,
    },
    Bin {
        left: Box<Node>,
        right: Box<Node>,
        op: Token,
    },
    Unary {
        value: Box<Node>,
        op: Token,
    },
    If {
        location: Token,
        logical: Box<Node>,
        body: Box<Node>,
        elseif: Option<Box<Node>>,
    },
    While {
        location: Token,
        logical: Box<Node>,
        body: Box<Node>,
    },
    Define {
        publicity: Publicity,
        name: Token,
        value: Box<Node>,
        typ: Option<TypePath>,
    },
    Assign {
        previous: Option<Box<Node>>,
        name: Token,
        value: Box<Node>,
    },
    Get {
        previous: Option<Box<Node>>,
        name: Token,
    },
    Call {
        previous: Option<Box<Node>>,
        name: Token,
        args: Vec<Node>,
    },
    FnDeclaration {
        name: Token,
        publicity: Publicity,
        params: Vec<Parameter>,
        body: Box<Node>,
        typ: Option<TypePath>,
    },
    Break {
        location: Token,
    },
    Continue {
        location: Token,
    },
    Use {
        location: Address,
        path: DependencyPath,
        name: Option<Token>,
    },
    Cond {
        left: Box<Node>,
        right: Box<Node>,
        op: Token,
    },
    Logical {
        left: Box<Node>,
        right: Box<Node>,
        op: Token,
    },
    Return {
        location: Token,
        value: Box<Node>,
    },
    TypeDeclaration {
        name: Token,
        publicity: Publicity,
        constructor: Vec<Parameter>,
        fields: Vec<Node>,
        functions: Vec<Node>,
    },
    For {
        iterable: Box<Node>,
        variable: Token,
        body: Box<Node>,
    },
    Range {
        location: Token,
        from: Box<Node>,
        to: Box<Node>,
    },
}

/// Ast tree
#[derive(Debug)]
pub struct Tree {
    pub body: Vec<Node>,
}
