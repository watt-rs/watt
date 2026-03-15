/// Imports
use crate::{Parser, errors::ParseError};
use watt_ast::{atom::TypeHint, expr::Expr, stmt::Stmt};
use watt_lex::token::TokenKind;
use watt_macros::bail;

/// Implementation
impl<'s> Parser<'s> {
    /// Let statement
    fn let_stmt(&mut self) -> Stmt {
        // Bumping `let`
        let start_span = self.peek().span.clone();
        self.bump();
        let name = self.expect(TokenKind::Id).lexeme;

        // Parsing hint
        let hint = if self.check(TokenKind::Colon) {
            self.type_hint()
        } else {
            TypeHint::Infer
        };

        // Parsing rhs
        self.expect(TokenKind::Eq);
        let expr = self.expr();
        let end_span = self.prev().span.clone();

        Stmt::Let(start_span + end_span, name, hint, expr)
    }

    /// Expression statement
    fn expr_stmt(&mut self) -> Stmt {
        let expr = self.expr();
        if self.check(TokenKind::Semi) {
            Stmt::Semi(expr)
        } else {
            Stmt::Expr(expr)
        }
    }

    /// Statement parsing with semicolon
    fn stmt(&mut self) -> Stmt {
        // Parsing statement kind
        let start_span = self.peek().span.clone();
        let stmt = match self.peek().kind {
            TokenKind::Let => self.let_stmt(),
            _ => self.expr_stmt(),
        };
        let end_span = self.prev().span.clone();
        // if `;` presented
        if self.check(TokenKind::Semi) {
            self.bump();
            stmt
        }
        // if not
        else {
            // if here's closing brace of the block, or the statement
            // does not need a semicolon, just returning it.
            if self.check(TokenKind::Rbrace) | !stmt.requires_semi() {
                stmt
            } else {
                bail!(ParseError::ExpectedSemicolon {
                    src: self.source.clone(),
                    span: (start_span + end_span).1.into()
                })
            }
        }
    }

    /// Block parsing
    pub fn block(&mut self) -> Expr {
        let start_span = self.peek().span.clone();

        let mut stmts = Vec::new();
        self.expect(TokenKind::Lbrace);
        while !self.check(TokenKind::Rbrace) {
            stmts.push(self.stmt());
        }
        self.expect(TokenKind::Rbrace);

        let end_span = self.prev().span.clone();

        Expr::Block(start_span + end_span, stmts)
    }
}
