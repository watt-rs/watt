/// Imports
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
        typ: Option<Token>,
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
        params: Vec<Token>,
        body: Box<Node>,
        typ: Option<Token>,
    },
    AnFnDeclaration {
        location: Token,
        params: Vec<Token>,
        body: Box<Node>,
        typ: Option<Token>,
    },
    Break {
        location: Token,
    },
    Continue {
        location: Token,
    },
    Use {
        path: Token,
        name: Token,
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
    Match {
        location: Token,
        matchable: Box<Node>,
        cases: Vec<MatchCase>,
        default: Box<Node>,
    },
    NewInstance {
        location: Token,
        typ: Token,
        constructor: Vec<Node>,
        should_push: bool,
    },
    Return {
        location: Token,
        value: Box<Node>,
    },
    Null {
        location: Token,
    },
    TypeDeclaration {
        name: Token,
        constructor: Vec<Token>,
        fields: Vec<Node>,
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
