/*
АСТ
 */
use std::collections::HashMap;
use crate::lexer::Token;

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
        previous: Box<Node>,
        name: Token,
        value: Box<Node>,
    },
    Assign {
        previous: Box<Node>,
        name: Token,
        value: Box<Node>,
    },
    Get {
        previous: Box<Node>,
        name: Token,
        should_push: bool,
    },
    Call {
        previous: Box<Node>,
        name: Token,
        args: Vec<Box<Node>>,
        should_push: bool,
    },
    FnDeclaration {
        name: Token,
        full_name: Token,
        args: Vec<Token>,
        body: Box<Node>,
    },
    AnFnDeclaration {
        args: Vec<Token>,
        body: Box<Node>,
    },
    Break {
        location: Token,
    },
    Continue {
        location: Token,
    },
    Import {
        imports: Vec<Token>,
    },
    List {
        location: Token,
        values: Vec<Box<Node>>,
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
        values: HashMap<Box<Node>, Box<Node>>,
    },
    Match {
        location: Token,
        matchable: Box<Node>,
        cases: Vec<Box<Node>>,
        default: Box<Node>,
    },
    Native {
        name: Token,
    },
    Instance {
        name: Token,
        constructor: Vec<Box<Node>>,
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
        fullname: Token,
        constructor: Vec<Token>,
        body: Box<Node>,
    },
    Unit {
        name: Token,
        fullname: Token,
        body: Box<Node>
    },
    For {
        iterable: Box<Node>,
        variable_name: Token,
    }
}