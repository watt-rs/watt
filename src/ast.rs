/*
АСТ
 */
use std::collections::HashMap;
use crate::address::Address;
use crate::lexer::Token;

pub enum Node {
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
        left: Node,
        right: Node,
        op: Token,
    },
    Unary {
        value: Node,
        op: Token
    },
    If {
        logical: Node,
        body: Node,
        elseif: Option<Node>
    },
    While {
        logical: Node,
        body: Node,
    },
    Define {
        name: Token,
        value: Node,
    },
    Assign {
        name: Token,
        value: Node,
    },
    Get {
        name: Token,
    },
    Call {
        name: Token,
        args: Vec<Node>,
    },
    FnDeclaration {
        name: Token,
        full_name: Token,
        args: Vec<Token>,
        body: Node,
    },
    AnFnDeclaration {
        args: Vec<Token>,
        body: Node,
    },
    Break {
        location: Address,
    },
    Continue {
        location: Address,
    },
    Import {
        imports: Vec<Token>,
    },
    List {
        location: Address,
        value: Vec<Node>,
    },
    Cond {
        left: Node,
        right: Node,
        op: Token,
    },
    Logical {
        left: Node,
        right: Node,
        op: Token,
    },
    Map {
        location: Address,
        value: HashMap<Node, Node>,
    },
    Match {
        matchable: Node,
        cases: Vec<Node>,
        default: Node,
    },
    Native {
        name: Token,
    },
    Instance {
        name: Token,
        constructor: Vec<Node>,
    },
    Ret {
        location: Address,
        value: Node,
    },
    Null {
        location: Address,
    },
    Type {
        name: Token,
        full_name: Token,
        constructor: Vec<Node>,
        body: Node,
    },
    Unit {
        name: Token,
        full_name: Token,
        body: Node
    },
    For {
        iterable: Node,
        variable_name: Token,
    }
}