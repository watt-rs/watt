use oil_common::address::Address;
//// Imports
use oil_lex::tokens::Token;

/// AST node
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
        name: Token,
        value: Box<Node>,
    },
    Assign {
        previous: Option<Box<Node>>,
        name: Token,
        value: Box<Node>,
    },
    Get {
        previous: Option<Box<Node>>,
        name: Token,
        should_push: bool,
    },
    Call {
        previous: Option<Box<Node>>,
        name: Token,
        args: Vec<Node>,
        should_push: bool,
    },
    FnDeclaration {
        name: Token,
        params: Vec<Token>,
        body: Box<Node>,
    },
    AnFnDeclaration {
        location: Token,
        params: Vec<Token>,
        body: Box<Node>,
    },
    Break {
        location: Token,
    },
    Continue {
        location: Token,
    },
    Import {
        location: Token,
        imports: Vec<Import>,
    },
    List {
        location: Token,
        values: Vec<Node>,
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
    Map {
        location: Token,
        values: Vec<(Node, Node)>,
    },
    Match {
        location: Token,
        matchable: Box<Node>,
        cases: Vec<MatchCase>,
        default: Box<Node>,
    },
    Native {
        name: Token,
        fn_name: Token,
    },
    Instance {
        location: Token,
        expr: Box<Node>,
        constructor: Vec<Node>,
        should_push: bool,
    },
    Ret {
        location: Token,
        value: Box<Node>,
    },
    Null {
        location: Token,
    },
    Type {
        name: Token,
        constructor: Vec<Token>,
        body: Box<Node>,
        impls: Vec<Token>,
    },
    Unit {
        name: Token,
        body: Box<Node>,
    },
    For {
        iterable: Box<Node>,
        variable_name: Token,
        body: Box<Node>,
    },
    Trait {
        name: Token,
        functions: Vec<TraitNodeFn>,
    },
    ErrorPropagation {
        location: Token,
        value: Box<Node>,
        should_push: bool,
    },
    Impls {
        location: Token,
        value: Box<Node>,
        expr: Box<Node>,
    },
    Range {
        location: Token,
        from: Box<Node>,
        to: Box<Node>,
    },
}

/// Import structure
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Import {
    pub addr: Address,
    pub file: String,
    pub variable: String,
}

/// Import implementation
impl Import {
    /// New import
    ///
    /// * `addr`: optional import address
    /// * `file`: import file
    /// * `variable`: import as
    ///
    pub fn new(addr: Address, file: String, variable: String) -> Self {
        Import {
            addr,
            file,
            variable,
        }
    }
}

/// Match statement case
/// Represents pattern value, body
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct MatchCase {
    pub value: Box<Node>,
    pub body: Box<Node>,
}

/// Match case implementation
impl MatchCase {
    /// New match case
    pub fn new(value: Box<Node>, body: Box<Node>) -> MatchCase {
        MatchCase { value, body }
    }
}

/// Trait node function
///
/// * `name`: name of fn
/// * `params`: fn params
/// * `default`: optional default impl
///
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TraitNodeFn {
    pub name: Token,
    pub params: Vec<Token>,
    pub default: Option<Box<Node>>,
}

/// Trait node function implementation
impl TraitNodeFn {
    pub fn new(name: Token, params: Vec<Token>, default: Option<Box<Node>>) -> Self {
        Self {
            name,
            params,
            default,
        }
    }
}

/// Sets should push for access node
pub fn set_should_push(node: Node, should_push: bool) -> Node {
    match node {
        Node::Get { previous, name, .. } => Node::Get {
            previous,
            name,
            should_push,
        },
        Node::Call {
            previous,
            name,
            args,
            ..
        } => Node::Call {
            previous,
            name,
            args,
            should_push,
        },
        Node::Instance {
            location,
            expr,
            constructor,
            ..
        } => Node::Instance {
            location,
            expr,
            constructor,
            should_push,
        },
        Node::ErrorPropagation {
            location, value, ..
        } => Node::ErrorPropagation {
            location,
            value,
            should_push,
        },
        _ => node,
    }
}
