/// Imports
use crate::Parser;
use watt_ast::atom::{Param, TypeHint};
use watt_lex::token::TokenKind;

/// Atoms parsing implementation
impl<'s> Parser<'s> {
    /// Parses generic args
    fn generic_args(&mut self) -> Vec<TypeHint> {
        if self.check(TokenKind::Lt) {
            self.sep_by(TokenKind::Lt, TokenKind::Gt, TokenKind::Comma, |p| {
                p.type_hint()
            })
        } else {
            Vec::new()
        }
    }

    /// Parses generic params
    pub(crate) fn generic_params(&mut self) -> Vec<String> {
        if self.check(TokenKind::Lt) {
            self.sep_by(TokenKind::Lt, TokenKind::Gt, TokenKind::Comma, |p| {
                p.expect(TokenKind::Id).lexeme
            })
        } else {
            Vec::new()
        }
    }

    /// Parses params
    pub(crate) fn params(&mut self) -> Vec<Param> {
        self.sep_by(
            TokenKind::Lparen,
            TokenKind::Rparen,
            TokenKind::Comma,
            |p| {
                let start_span = p.peek().span.clone();
                let name = p.expect(TokenKind::Id).lexeme;
                p.expect(TokenKind::Colon);
                let hint = p.type_hint();
                let end_span = p.prev().span.clone();

                Param {
                    span: start_span + end_span,
                    name,
                    hint,
                }
            },
        )
    }

    /// Parses id type hint
    fn id_type_hint(&mut self) -> TypeHint {
        // bumping id
        let start_span = self.peek().span.clone();
        let id = self.bump().lexeme;

        // if dot presented
        if self.check(TokenKind::Dot) {
            self.bump();

            let name = self.expect(TokenKind::Id).lexeme;
            let args = self.generic_args();
            let end_span = self.prev().span.clone();

            TypeHint::Module {
                span: start_span + end_span,
                module: id,
                name: name,
                args,
            }
        }
        // If not
        else {
            let args = self.generic_args();
            let end_span = self.prev().span.clone();

            TypeHint::Local {
                span: start_span + end_span,
                name: id,
                args,
            }
        }
    }

    /// Parses function type hint
    fn fn_type_hint(&mut self) -> TypeHint {
        // bumping `fn`
        let start_span = self.peek().span.clone();
        self.bump();

        // parsing params
        let params = self.sep_by(
            TokenKind::Lparen,
            TokenKind::Rparen,
            TokenKind::Comma,
            |p| p.type_hint(),
        );

        // parsing return type
        let ret = if self.check(TokenKind::Colon) {
            self.bump();
            Box::new(self.type_hint())
        } else {
            Box::new(TypeHint::Infer)
        };
        let end_span = self.prev().span.clone();

        TypeHint::Function {
            span: start_span + end_span,
            params,
            ret,
        }
    }

    /// Parses type hint
    pub(crate) fn type_hint(&mut self) -> TypeHint {
        if self.check(TokenKind::Fn) {
            self.fn_type_hint()
        } else {
            self.id_type_hint()
        }
    }
}
