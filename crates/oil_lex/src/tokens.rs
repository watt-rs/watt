//// Imports
use ecow::EcoString;
use oil_common::address::Address;

/// Token kind
#[derive(Debug, Clone, Eq, PartialEq, Copy, Hash)]
#[allow(dead_code)]
pub enum TokenKind {
    Let,           // let
    Fn,            // fn
    Plus,          // +
    Minus,         // -
    Star,          // *
    Slash,         // /
    Percent,       // %
    Caret,         // ^
    Or,            // || | or
    And,           // && | and
    BitwiseOr,     // |
    BitwiseAnd,    // &
    AddAssign,     // +=
    SubAssign,     // -=
    MulAssign,     // *-
    DivAssign,     // /=
    AndEq,         // &=
    OrEq,          // |=
    XorEq,         // ^=
    Lparen,        // (
    Rparen,        // )
    Lbrace,        // {
    Rbrace,        // }
    Eq,            // ==
    NotEq,         // !=
    Text,          // 'text'
    Number,        // 1234567890.0123456789
    Assign,        // =
    Id,            // variable id
    Comma,         // ,
    Ret,           // return
    Done,          // done
    If,            // if
    Bool,          // bool
    While,         // while
    Type,          // type
    Enum,          // enum
    Dot,           // dot
    Greater,       // >
    Less,          // <
    GreaterEq,     // >=
    LessEq,        // <=
    Concat,        // <>
    Elif,          // elif
    Else,          // else
    Use,           // use
    Break,         // break
    Lbracket,      // [
    Rbracket,      // ]
    Colon,         // :
    PathSeparator, // ::
    For,           // for
    Bang,          // !
    In,            // in
    Continue,      // continue
    Unit,          // unit
    Range,         // ..
    As,            // as
    Pub,           // pub
    Match,         // match
    Arrow,         // arrow
    Extern,        // extern
}

/// Token structure
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Token {
    pub tk_type: TokenKind,
    pub value: EcoString,
    pub address: Address,
}

/// Token implementation
impl Token {
    /// Creates token from tk_type, value, address
    pub fn new(tk_type: TokenKind, value: EcoString, address: Address) -> Token {
        Token {
            tk_type,
            value,
            address,
        }
    }
}
