/// Imports
use ecow::EcoString;
use oil_common::address::Address;
use oil_lex::tokens::Token;

/// Dependency path
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DependencyPath {
    pub address: Address,
    pub module: EcoString,
}

/// Type path
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TypePath {
    Local {
        location: Address,
        name: EcoString,
    },
    Module {
        location: Address,
        module: EcoString,
        name: EcoString,
    },
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

/// Enum constructor
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct EnumConstructor {
    pub name: Token,
    pub params: Vec<Parameter>,
}

/// Enum constructor implementation
impl EnumConstructor {
    /// Creates new parameter
    pub fn new(name: Token, params: Vec<Parameter>) -> Self {
        Self { name, params }
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
        location: Address,
        what: Box<Node>,
        value: Box<Node>,
    },
    Get {
        name: Token,
    },
    FieldAccess {
        container: Box<Node>,
        name: Token,
    },
    Call {
        location: Address,
        what: Box<Node>,
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
        location: Address,
        name: Token,
        publicity: Publicity,
        constructor: Vec<Parameter>,
        fields: Vec<Node>,
        functions: Vec<Node>,
    },
    EnumDeclaration {
        location: Address,
        name: Token,
        publicity: Publicity,
        variants: Vec<EnumConstructor>,
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
