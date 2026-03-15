/// Imports
use crate::{Parser, errors::ParseError};
use watt_ast::{
    atom::{BinOp, Lit, UnOp},
    expr::{Case, Expr},
};
use watt_lex::token::TokenKind;
use watt_macros::bail;

/// Exprs parsing implementation
impl<'s> Parser<'s> {
    /// Group `( expr )` expression parsing
    fn group(&mut self) -> Expr {
        let start_span = self.peek().span.clone();
        self.expect(TokenKind::Lparen);
        let expr = self.expr();
        self.expect(TokenKind::Rparen);
        let end_span = self.prev().span.clone();

        Expr::Paren(start_span + end_span, Box::new(expr))
    }

    /// Variable parsing
    fn variable(&mut self) -> Expr {
        // Parsing base identifier
        let start_span = self.peek().span.clone();
        let id = self.expect(TokenKind::Id).lexeme;

        // Result node
        let mut result = Expr::Var(start_span.clone(), id);

        // Checking for dots and parens
        loop {
            // Checking for chain `a.b.c.d`
            if self.check(TokenKind::Dot) {
                self.bump();

                let id = self.expect(TokenKind::Id).lexeme;
                let end_span = self.prev().span.clone();

                result = Expr::Suffix(start_span.clone() + end_span, Box::new(result), id);
                continue;
            }

            // Checking for start of call with `(`
            if self.check(TokenKind::Lparen) {
                let args = self.sep_by(
                    TokenKind::Lparen,
                    TokenKind::Rparen,
                    TokenKind::Comma,
                    |p| p.expr(),
                );
                let end_span = self.prev().span.clone();

                result = Expr::Call(start_span.clone() + end_span, Box::new(result), args);
                continue;
            }

            // Breaking loop
            break;
        }
        result
    }

    /// If expression parsing
    fn if_expr(&mut self) -> Expr {
        // Bumping `if`
        let start_span = self.peek().span.clone();
        self.bump();

        // Parsing if block
        let expr = self.expr();
        let block = self.block();

        // Parsing else branch, if exists
        if self.check(TokenKind::Else) {
            self.bump();

            let branch = if self.check(TokenKind::If) {
                self.if_expr()
            } else {
                self.block()
            };

            let end_span = self.prev().span.clone();
            Expr::If(
                start_span + end_span,
                Box::new(expr),
                Box::new(block),
                Some(Box::new(branch)),
            )
        } else {
            let end_span = self.prev().span.clone();
            Expr::If(start_span + end_span, Box::new(expr), Box::new(block), None)
        }
    }

    /// Function expression parsing
    fn fn_expr(&mut self) -> Expr {
        // Bumping `fn`
        let start_span = self.peek().span.clone();
        self.bump();

        // Collecting params
        let params = self.sep_by(
            TokenKind::Lparen,
            TokenKind::Rparen,
            TokenKind::Comma,
            |p| p.expect(TokenKind::Id).lexeme,
        );

        // Parsing block or expr body
        let body = if self.check(TokenKind::Lbrace) {
            self.block()
        } else {
            self.expr()
        };
        let end_span = self.prev().span.clone();

        Expr::Function(start_span + end_span, params, Box::new(body))
    }

    /// Case parsing
    fn case(&mut self) -> Case {
        // Patterns of the case
        let start_span = self.peek().span.clone();
        let pats = self.sep_by_2(TokenKind::Comma, |p| p.pat());

        // -> { body, ... }
        self.expect(TokenKind::Colon);
        let body = if self.check(TokenKind::Lbrace) {
            self.block()
        } else {
            self.expr()
        };
        let end_span = self.prev().span.clone();

        Case {
            span: start_span + end_span,
            pats,
            body,
        }
    }

    /// Match expression parsing
    fn match_expr(&mut self) -> Expr {
        // `match value { patterns, ... }`
        let start_span = self.peek().span.clone();
        self.expect(TokenKind::Match);
        let value = self.expr();

        // Parsing cases
        let cases = self.sep_by(
            TokenKind::Lbrace,
            TokenKind::Rbrace,
            TokenKind::Comma,
            |p| p.case(),
        );
        let end_span = self.prev().span.clone();

        Expr::Match(start_span + end_span, Box::new(value), cases)
    }

    /// Atom expression parsing
    fn atom(&mut self) -> Expr {
        let tk = self.peek().clone();
        match tk.kind {
            // Literals parsing
            TokenKind::Lparen => self.group(),
            TokenKind::Number => {
                self.bump();
                if tk.lexeme.contains(".") {
                    Expr::Lit(tk.span, Lit::Float(tk.lexeme))
                } else {
                    Expr::Lit(tk.span, Lit::Int(tk.lexeme))
                }
            }
            TokenKind::String => {
                self.bump();
                Expr::Lit(tk.span, Lit::String(tk.lexeme))
            }
            TokenKind::Bool => {
                self.bump();
                Expr::Lit(tk.span, Lit::Bool(tk.lexeme))
            }
            // Variable parsing
            TokenKind::Id => self.variable(),
            // Branch expressions parsing
            TokenKind::If => self.if_expr(),
            TokenKind::Match => self.match_expr(),
            // Function parsing
            TokenKind::Fn => self.fn_expr(),
            _ => bail!(ParseError::UnexpectedExprToken {
                got: tk.kind,
                src: self.source.clone(),
                span: tk.span.1.into(),
            }),
        }
    }

    /// Unary expression parsing
    fn unary_expr(&mut self) -> Expr {
        if self.check(TokenKind::Minus) || self.check(TokenKind::Bang) {
            let start_span = self.peek().span.clone();

            let op = match self.bump().kind {
                TokenKind::Minus => UnOp::Neg,
                TokenKind::Bang => UnOp::Bang,
                _ => unreachable!(),
            };

            let value = self.unary_expr();
            let end_span = self.prev().span.clone();

            return Expr::Unary(start_span + end_span, Box::new(value), op);
        }

        self.atom()
    }

    /// Factor expression parsing
    fn factor_expr(&mut self) -> Expr {
        let start_span = self.peek().span.clone();
        let mut left = self.unary_expr();

        while self.check(TokenKind::Star)
            || self.check(TokenKind::Slash)
            || self.check(TokenKind::Percent)
        {
            let op = match self.bump().kind {
                TokenKind::Star => BinOp::Mul,
                TokenKind::Slash => BinOp::Div,
                TokenKind::Percent => BinOp::Mod,
                _ => unreachable!(),
            };

            let right = self.unary_expr();
            let end_span = self.prev().span.clone();

            left = Expr::Bin(
                start_span.clone() + end_span,
                Box::new(left),
                Box::new(right),
                op,
            );
        }

        left
    }

    /// Term expression parsing
    fn term_expr(&mut self) -> Expr {
        let start_span = self.peek().span.clone();
        let mut left = self.factor_expr();

        while self.check(TokenKind::Plus) || self.check(TokenKind::Minus) {
            let op = match self.bump().kind {
                TokenKind::Plus => BinOp::Add,
                TokenKind::Minus => BinOp::Sub,
                _ => unreachable!(),
            };

            let right = self.factor_expr();
            let end_span = self.prev().span.clone();

            left = Expr::Bin(
                start_span.clone() + end_span,
                Box::new(left),
                Box::new(right),
                op,
            );
        }

        left
    }

    /// Compare expression parsing
    fn compare_expr(&mut self) -> Expr {
        let start_span = self.peek().span.clone();
        let mut left = self.term_expr();

        while self.check(TokenKind::Ge)
            || self.check(TokenKind::Gt)
            || self.check(TokenKind::Le)
            || self.check(TokenKind::Lt)
        {
            let op = match self.bump().kind {
                TokenKind::Ge => BinOp::Ge,
                TokenKind::Gt => BinOp::Gt,
                TokenKind::Le => BinOp::Le,
                TokenKind::Lt => BinOp::Lt,
                _ => unreachable!(),
            };

            let right = self.factor_expr();
            let end_span = self.prev().span.clone();

            left = Expr::Bin(
                start_span.clone() + end_span,
                Box::new(left),
                Box::new(right),
                op,
            );
        }

        left
    }

    /// Equality expression parsing
    fn equality_expr(&mut self) -> Expr {
        let start_span = self.peek().span.clone();
        let mut left = self.compare_expr();

        while self.check(TokenKind::DoubleEq) || self.check(TokenKind::BangEq) {
            let op = match self.bump().kind {
                TokenKind::DoubleEq => BinOp::Eq,
                TokenKind::BangEq => BinOp::Ne,
                _ => unreachable!(),
            };

            let right = self.compare_expr();
            let end_span = self.prev().span.clone();

            left = Expr::Bin(
                start_span.clone() + end_span,
                Box::new(left),
                Box::new(right),
                op,
            );
        }

        left
    }

    /// `bitwise and` expression parsing
    fn bit_and_expr(&mut self) -> Expr {
        let start_span = self.peek().span.clone();
        let mut left = self.equality_expr();

        while self.check(TokenKind::Ampersand) {
            self.bump();

            let right = self.equality_expr();
            let end_span = self.prev().span.clone();

            left = Expr::Bin(
                start_span.clone() + end_span,
                Box::new(left),
                Box::new(right),
                BinOp::BitAnd,
            );
        }

        left
    }

    /// `bitwise xor` expression parsing
    fn bit_xor_expr(&mut self) -> Expr {
        let start_span = self.peek().span.clone();
        let mut left = self.bit_and_expr();

        while self.check(TokenKind::Caret) {
            self.bump();

            let right = self.bit_and_expr();
            let end_span = self.prev().span.clone();

            left = Expr::Bin(
                start_span.clone() + end_span,
                Box::new(left),
                Box::new(right),
                BinOp::Xor,
            );
        }

        left
    }

    /// `bitwise or` expression parsing
    fn bit_or_expr(&mut self) -> Expr {
        let start_span = self.peek().span.clone();
        let mut left = self.bit_xor_expr();

        while self.check(TokenKind::Bar) {
            self.bump();

            let right = self.bit_xor_expr();
            let end_span = self.prev().span.clone();

            left = Expr::Bin(
                start_span.clone() + end_span,
                Box::new(left),
                Box::new(right),
                BinOp::BitOr,
            );
        }

        left
    }

    /// `Logical and` expression parsing
    fn logical_and_expr(&mut self) -> Expr {
        let start_span = self.peek().span.clone();
        let mut left = self.bit_or_expr();

        while self.check(TokenKind::DoubleAmp) {
            self.bump();

            let right = self.bit_or_expr();
            let end_span = self.prev().span.clone();

            left = Expr::Bin(
                start_span.clone() + end_span,
                Box::new(left),
                Box::new(right),
                BinOp::And,
            );
        }

        left
    }

    /// `Logical or` expression parsing
    fn logical_or_expr(&mut self) -> Expr {
        let start_span = self.peek().span.clone();
        let mut left = self.logical_and_expr();

        while self.check(TokenKind::DoubleBar) {
            self.bump();

            let right = self.logical_and_expr();
            let end_span = self.prev().span.clone();

            left = Expr::Bin(
                start_span.clone() + end_span,
                Box::new(left),
                Box::new(right),
                BinOp::Or,
            );
        }

        left
    }

    /// `Assign` expression parsing
    fn assign_expr(&mut self) -> Expr {
        let start_span = self.peek().span.clone();
        let mut left = self.logical_or_expr();

        while self.check(TokenKind::Eq) {
            self.bump();

            let right = self.logical_or_expr();
            let end_span = self.prev().span.clone();

            left = Expr::Assign(
                start_span.clone() + end_span,
                Box::new(left),
                Box::new(right),
            );
        }

        left
    }

    /// Parses expression
    pub fn expr(&mut self) -> Expr {
        self.assign_expr()
    }
}
