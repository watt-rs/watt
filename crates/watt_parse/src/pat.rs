/// Imports
use crate::{Parser, errors::ParseError};
use watt_ast::{
    atom::Lit,
    expr::{Expr, Pat, UnpackParam},
};
use watt_lex::token::{Span, Token, TokenKind};
use watt_macros::bail;

/// Patterns parsing implementation
impl<'s> Parser<'s> {
    /// Enum pattern name parsing
    fn enum_pat_name(&mut self, id: Token) -> Expr {
        // Parsing base identifier
        let start_span = id.span.clone();

        // Result node
        let mut result = Expr::Var(start_span.clone(), id.lexeme);

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

            // Breaking loop
            break;
        }
        result
    }

    /// Unpack pattern parsing
    fn unpack_pat(&mut self, start_span: Span, en: Expr) -> Pat {
        // Parsing params
        let params = self.sep_by(
            TokenKind::Lparen,
            TokenKind::Rparen,
            TokenKind::Comma,
            |p| {
                if p.check(TokenKind::Wildcard) {
                    p.bump();
                    UnpackParam::Wildcard
                } else {
                    let id = p.expect(TokenKind::Id).lexeme;
                    UnpackParam::Bind(id)
                }
            },
        );

        let end_span = self.prev().span.clone();
        Pat::Unpack(start_span + end_span, en, params)
    }

    /// Signle pattern parsing
    fn single_pat(&mut self) -> Pat {
        let tk = self.bump();
        match tk.kind {
            // Literal pattern
            TokenKind::String => Pat::Lit(tk.span, Lit::String(tk.lexeme)),
            TokenKind::Bool => Pat::Lit(tk.span, Lit::Bool(tk.lexeme)),
            TokenKind::Number => {
                if tk.lexeme.contains(".") {
                    Pat::Lit(tk.span, Lit::Bool(tk.lexeme))
                } else {
                    Pat::Lit(tk.span, Lit::Int(tk.lexeme))
                }
            }
            // Wilcard pattern
            TokenKind::Wildcard => Pat::Wildcard,
            // Identifier pattern
            TokenKind::Id => {
                // If dot presented -> enum pattern
                if self.check(TokenKind::Dot) {
                    // Parsing enum pattern name
                    let en = self.enum_pat_name(tk);

                    // If paren presents -> unpack pattern
                    if self.check(TokenKind::Lparen) {
                        self.unpack_pat(en.span(), en)
                    }
                    // If not -> variant pattern
                    else {
                        let end_span = self.prev().span.clone();
                        Pat::Variant(en.span() + end_span, en)
                    }
                }
                // If not -> bind pattern
                else {
                    let binding = self.expect(TokenKind::Id).lexeme;
                    let end_span = self.prev().span.clone();

                    Pat::BindTo(tk.span + end_span, binding)
                }
            }
            // Otherwise, bailing error
            got => bail!(ParseError::UnexpectedPatToken {
                got,
                src: tk.span.0,
                span: tk.span.1.into()
            }),
        }
    }

    /// Pattern parsing
    pub(crate) fn pat(&mut self) -> Pat {
        // parsing single pattern
        let pat = self.single_pat();

        // cecking if more patterns presented
        if self.check(TokenKind::Bar) {
            // parsing `or` pattern
            self.expect(TokenKind::Bar);

            // left and right pattern
            let a = Box::new(pat);
            let b = Box::new(self.pat());

            Pat::Or(a, b)
        } else {
            pat
        }
    }
}
