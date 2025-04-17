/*
АСТ
 */
use std::collections::{BTreeMap, HashMap};
use crate::address::Address;
use crate::errors::{Error, ErrorType};
use crate::import::Import;
use crate::lexer::Token;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Node {
    Block {
        body: Vec<Box<Node>>,
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
        args: Vec<Box<Node>>,
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
        imports: Vec<Import>,
    },
    List {
        location: Token,
        values: Box<Vec<Box<Node>>>,
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
        cases: Vec<Box<Node>>,
        default: Box<Node>,
    },
    Native {
        name: Token,
        fn_name: Token
    },
    Instance {
        name: Token,
        constructor: Vec<Box<Node>>,
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
    }
}

pub fn set_should_push(node: Node, should_push: bool, address: Address) -> Result<Node, Error> {
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
        _ => {
            Err(Error::new(
                ErrorType::Parsing,
                address,
                format!("couldn't apply should_push changes to: {:?}", node).to_string(),
                "check your code.".to_string()
            ))
        }
    }
}