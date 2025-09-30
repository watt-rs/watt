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
    Function {
        location: Address,
        params: Vec<TypePath>,
        ret: Box<TypePath>,
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

/// Pattern
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Pattern {
    // Unwrap enum pattern
    // `Pot.Full { flower, .. }`
    Unwrap { en: Node, fields: Vec<Token> },
    // `123456`
    Value(Node),
    // `0..10`
    Range { start: Node, end: Node },
}

/// Case
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Case {
    pub address: Address,
    pub pattern: Pattern,
    pub body: Node,
}

/// Use kind
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum UseKind {
    AsName(Token),
    ForNames(Vec<Token>),
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
        location: Address,
        logical: Box<Node>,
        body: Box<Node>,
        elseif: Option<Box<Node>>,
    },
    While {
        location: Address,
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
    AnonymousFn {
        location: Address,
        params: Vec<Parameter>,
        body: Box<Node>,
        typ: Option<TypePath>,
    },
    Break {
        location: Address,
    },
    Continue {
        location: Address,
    },
    Use {
        location: Address,
        path: DependencyPath,
        kind: UseKind,
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
        location: Address,
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
    Match {
        location: Address,
        value: Box<Node>,
        cases: Vec<Case>,
    },
    Range {
        location: Address,
        from: Box<Node>,
        to: Box<Node>,
    },
    ExternFn {
        location: Address,
        name: Token,
        publicity: Publicity,
        params: Vec<Parameter>,
        typ: Option<TypePath>,
        body: Token,
    },
}

/// Ast tree
#[derive(Debug)]
pub struct Tree {
    pub body: Vec<Node>,
}
