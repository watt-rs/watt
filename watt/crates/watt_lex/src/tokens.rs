// imports
use watt_common::address::Address;

/// Token kind
#[derive(Debug, Clone, Eq, PartialEq, Copy, Hash)]
#[allow(dead_code)]
pub enum TokenKind {
    Fn,
    Op,        // +, -, *, /
    Lparen,    // (
    Rparen,    // )
    Lbrace,    // {
    Rbrace,    // }
    Lambda,    // lambda
    Walrus,    // :=
    Eq,        // ==
    NotEq,     // !=
    Text,      // 'text'
    Number,    // 1234567890.0123456789
    Assign,    // =
    Id,        // variable id
    Comma,     // ,
    Ret,       // return
    If,        // if
    Bool,      // bool
    While,     // while
    Type,      // type
    New,       // new
    Dot,       // dot
    Greater,   // >
    Less,      // <
    GreaterEq, // >=
    LessEq,    // <=
    Null,      // null
    Elif,      // elif
    Else,      // else
    And,       // logical and
    Or,        // logical or
    Import,    // import
    AssignAdd, // assign add
    AssignSub, // assign sub
    AssignMul, // assign mul
    AssignDiv, // assign divide
    Break,     // break
    AssignAnd, // &
    AssignOr,  // |
    AssignXor, // ^
    Match,     // match
    Case,      // case
    Default,   // default
    Lbracket,  // [
    Rbracket,  // ]
    Colon,     // colon :
    For,       // for
    Bang,      // !
    In,        // in
    Continue,  // continue
    Arrow,     // ->
    Unit,      // unit
    Native,    // native
    With,      // with
    Trait,     // trait
    Impl,      // impl
    Question,  // ?
    Impls,     // impls
    Range,     // ..
}

/// Token structure
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Token {
    pub tk_type: TokenKind,
    pub value: String,
    pub address: Address,
}
/// Token implementation
impl Token {
    /// Creates token from tk_type, value, address
    pub fn new(tk_type: TokenKind, value: String, address: Address) -> Token {
        Token {
            tk_type,
            value,
            address,
        }
    }
}
