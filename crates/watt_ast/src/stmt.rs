/// Imports
use crate::{atom::TypeHint, expr::Expr};
use watt_lex::token::Span;

/// Statement kind
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Stmt {
    /// Let definition
    Let(Span, String, TypeHint, Expr),

    /// Expr without trailing semi-colon
    Expr(Expr),

    /// Expr with trailing semi-colon
    Semi(Expr),
}

/// Implementation
impl Stmt {
    /// Returns true if statement requires semicolon after it
    pub fn requires_semi(&self) -> bool {
        match self {
            Stmt::Let(_, _, _, _) | Stmt::Semi(_) => true,
            Stmt::Expr(_) => false,
        }
    }
}
