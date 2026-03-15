/// Imports
use crate::{
    atom::{BinOp, Lit, UnOp},
    stmt::Stmt,
};
use watt_lex::token::Span;

/// Represents unpack pattern param
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UnpackParam {
    /// Binding to a variable
    Bind(String),

    /// No binding
    Wildcard,
}

/// Represents pattern
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Pat {
    /// Represents literal pattern, e.g `123`
    Lit(Span, Lit),

    /// Represents just enum variant pattern
    Variant(Span, Expr),

    /// Represents enum fields unpack pattern
    Unpack(Span, Expr, Vec<UnpackParam>),

    /// Represents bind pattern
    BindTo(Span, String),

    /// Represents wildcard pattern
    Wildcard,

    /// Represents or pattern
    Or(Box<Pat>, Box<Pat>),
}

/// Represents case
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Case {
    pub span: Span,
    pub pats: Vec<Pat>,
    pub body: Expr,
}

/// Represents expression
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    /// Literal expression
    Lit(Span, Lit),

    /// Represents todo expression (e.g `todo as "simple todo"`)
    Todo(Span, Option<String>),

    /// Represents panic expression (e.g `panic as "simple panic"`)
    Panic(Span, Option<String>),

    /// Represents unary expression
    Unary(Span, Box<Expr>, UnOp),

    /// Represents binary expression
    Bin(Span, Box<Expr>, Box<Expr>, BinOp),

    /// Assignment expression
    Assign(Span, Box<Expr>, Box<Expr>),

    /// Represents if expression (cond, then, else)
    If(Span, Box<Expr>, Box<Expr>, Option<Box<Expr>>),

    /// Represents variable access
    Var(Span, String),

    /// Represents field access
    Suffix(Span, Box<Expr>, String),

    /// Represents call expression
    Call(Span, Box<Expr>, Vec<Expr>),

    /// Represents anonymous function expression
    Function(Span, Vec<String>, Box<Expr>),

    /// Represents match expression
    Match(Span, Box<Expr>, Vec<Case>),

    /// Represents paren expression
    Paren(Span, Box<Expr>),

    /// Block expression
    Block(Span, Vec<Stmt>),

    /// None expression
    None(Span),
}

/// Implementation
impl Expr {
    /// Returns expression span
    pub fn span(&self) -> Span {
        match self {
            Expr::Lit(span, _) => span.clone(),
            Expr::Todo(span, _) => span.clone(),
            Expr::Panic(span, _) => span.clone(),
            Expr::Unary(span, _, _) => span.clone(),
            Expr::Bin(span, _, _, _) => span.clone(),
            Expr::Assign(span, _, _) => span.clone(),
            Expr::If(span, _, _, _) => span.clone(),
            Expr::Var(span, _) => span.clone(),
            Expr::Suffix(span, _, _) => span.clone(),
            Expr::Call(span, _, _) => span.clone(),
            Expr::Function(span, _, _) => span.clone(),
            Expr::Match(span, _, _) => span.clone(),
            Expr::Paren(span, _) => span.clone(),
            Expr::Block(span, _) => span.clone(),
            Expr::None(span) => span.clone(),
        }
    }
}
