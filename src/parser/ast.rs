// импорты
use crate::lexer::lexer::*;
use crate::parser::import::Import;
use crate::errors::errors::Error;

// нода
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
        op: Token
    },
    If {
        location: Token,
        logical: Box<Node>,
        body: Box<Node>,
        elseif: Option<Box<Node>>
    },
    While {
        location: Token,
        logical: Box<Node>,
        body: Box<Node>,
    },
    Define {
        previous: Option<Box<Node>>,
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
        full_name: Option<Token>,
        params: Vec<Token>,
        body: Box<Node>,
    },
    AnFnDeclaration {
        location: Token, // .
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
        values: Box<Vec<(Box<Node>, Box<Node>)>>,
    },
    Match {
        location: Token,
        matchable: Box<Node>,
        cases: Vec<Node>,
        default: Box<Node>,
    },
    Native {
        name: Token,
        fn_name: Token
    },
    Instance {
        name: Token,
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
        full_name: Option<Token>,
        constructor: Vec<Token>,
        body: Box<Node>,
        impls: Vec<Token>
    },
    Unit {
        name: Token,
        full_name: Option<Token>,
        body: Box<Node>
    },
    For {
        iterable: Box<Node>,
        variable_name: Token,
        body: Box<Node>
    },
    Trait {
        name: Token,
        full_name: Option<Token>,
        functions: Vec<TraitNodeFn>
    },
    ErrorPropagation {
        location: Token,
        value: Box<Node>,
        should_push: bool,
    },
    Impls {
        value: Box<Node>,
        trait_name: Token
    },
    Range {
        location: Token,
        from: Box<Node>,
        to: Box<Node>
    }
}

// функция трейта
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TraitNodeFn {
    pub name: Token,
    pub params: Vec<Token>,
    pub default: Option<Box<Node>>
}
impl TraitNodeFn {
    pub fn new(name: Token, params: Vec<Token>, default: Option<Box<Node>>) -> Self {
        Self { name, params, default }
    }
}

// установка should_push
pub fn set_should_push(node: Node, should_push: bool) -> Result<Node, Error> {
    match node {
        Node::Get { previous, name, .. } => {
            Ok(Node::Get {
                previous,
                name,
                should_push,
            })
        },
        Node::Call { previous, name, args, .. } => {
            Ok(Node::Call {
                previous,
                name,
                args,
                should_push,
            })
        },
        Node::Instance { name, constructor, .. } => {
            Ok(Node::Instance {
                name,
                constructor,
                should_push,
            })
        },
        Node::ErrorPropagation { location, value, .. } => {
            Ok(Node::ErrorPropagation {
                location,
                value,
                should_push,
            })
        }
        _ => {
            Ok(node)
        }
    }
}