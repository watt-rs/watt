/// Imports
use crate::errors::ParseError;
use miette::NamedSource;
use std::sync::Arc;
use watt_ast::ast::*;
use watt_common::{bail, skip};
use watt_lex::tokens::{Token, TokenKind};

/// Parser structure
pub struct Parser<'file> {
    /// Tokens buffer
    tokens: Vec<Token>,
    /// Current index
    pub(crate) current: u128,
    /// Source file
    pub(crate) source: &'file Arc<NamedSource<String>>,
}

/// Parser implementation
#[allow(unused_qualifications)]
impl<'file> Parser<'file> {
    /// New parser
    pub fn new(tokens: Vec<Token>, source: &'file Arc<NamedSource<String>>) -> Self {
        Parser {
            tokens,
            current: 0,
            source,
        }
    }

    /// Parsing all declarations
    pub fn parse(&mut self) -> Module {
        // parsing declaration before reaching
        // end of file
        let mut declarations: Vec<Declaration> = Vec::new();
        let mut dependencies: Vec<Dependency> = Vec::new();
        while !self.is_at_end() {
            match self.peek().tk_type {
                TokenKind::Pub => {
                    self.consume(TokenKind::Pub);
                    declarations.push(self.declaration(Publicity::Public))
                }
                TokenKind::Use => dependencies.push(self.use_declaration()),
                _ => declarations.push(self.declaration(Publicity::Private)),
            }
        }

        Module {
            source: self.source.to_owned(),
            dependencies,
            declarations,
        }
    }

    /// Block parsing
    pub(crate) fn block(&mut self) -> Block {
        // parsing statement before reaching
        // end of file, or a `}`
        let mut nodes: Vec<Statement> = Vec::new();
        let start_location = self.peek().address.clone();
        self.consume(TokenKind::Lbrace);
        while !self.check(TokenKind::Rbrace) {
            nodes.push(self.statement());
        }
        self.consume(TokenKind::Rbrace);
        let end_location = self.previous().address.clone();

        Block {
            location: start_location + end_location,
            body: nodes,
        }
    }

    /// Block or expr parsing
    pub(crate) fn block_or_expr(&mut self) -> Either<Block, Expression> {
        // if lbrace passed
        if self.check(TokenKind::Lbrace) {
            // parsing statement before reaching
            // end of file, or a `}`
            let mut nodes: Vec<Statement> = Vec::new();
            self.consume(TokenKind::Lbrace);
            let start_location = self.peek().address.clone();
            while !self.check(TokenKind::Rbrace) {
                nodes.push(self.statement());
            }
            let end_location = self.previous().address.clone();
            self.consume(TokenKind::Rbrace);
            Either::Left(Block {
                location: start_location + end_location,
                body: nodes,
            })
        } else {
            // `=`
            self.consume(TokenKind::Assign);
            // parsing single expression
            Either::Right(self.expr())
        }
    }

    /// Block or box expr parsing
    pub(crate) fn block_or_box_expr(&mut self) -> Either<Block, Box<Expression>> {
        // if lbrace passed
        if self.check(TokenKind::Lbrace) {
            // parsing statement before reaching
            // end of file, or a `}`
            let mut nodes: Vec<Statement> = Vec::new();
            self.consume(TokenKind::Lbrace);
            let start_location = self.peek().address.clone();
            while !self.check(TokenKind::Rbrace) {
                nodes.push(self.statement());
            }
            let end_location = self.previous().address.clone();
            self.consume(TokenKind::Rbrace);
            Either::Left(Block {
                location: start_location + end_location,
                body: nodes,
            })
        } else {
            // `=`
            self.consume(TokenKind::Assign);
            // parsing single expression
            Either::Right(Box::new(self.expr()))
        }
    }

    /// Checks expression is const
    pub(crate) fn check_value_const(&mut self, expr: &Expression) {
        #[allow(unused_variables)]
        match expr {
            // expressions that depedends on variables
            // or logical clauses are non-const by default.
            Expression::PrefixVar { location, .. }
            | Expression::SuffixVar { location, .. }
            | Expression::Call { location, .. }
            | Expression::Function { location, .. }
            | Expression::Match { location, .. }
            | Expression::Todo { location, .. }
            | Expression::Panic { location, .. }
            | Expression::If { location, .. } => bail!(ParseError::NonConstExpr {
                src: self.source.clone(),
                span: location.span.clone().into(),
            }),
            // `literals` are const by default.
            Expression::Int { location, .. }
            | Expression::Float { location, .. }
            | Expression::String { location, .. }
            | Expression::Bool { location, .. } => skip!(),
            // `binary`, `as` and `unary` operations need to be checked.
            Expression::Bin {
                location,
                left,
                right,
                op,
            } => {
                self.check_value_const(left);
                self.check_value_const(right);
            }
            Expression::As {
                location,
                value,
                typ,
            } => self.check_value_const(value),
            Expression::Unary {
                location,
                value,
                op,
            } => {
                self.check_value_const(value);
            }
        }
    }

    /*
     helper functions
    */

    /// Gets current token, then adds 1 to current.
    pub(crate) fn advance(&mut self) -> &Token {
        match self.tokens.get(self.current as usize) {
            Some(tk) => {
                self.current += 1;
                tk
            }
            None => bail!(ParseError::UnexpectedEof),
        }
    }

    /// Consumes token by kind, if expected kind doesn't equal
    /// current token kind - raises error.
    pub(crate) fn consume(&mut self, tk_type: TokenKind) -> &Token {
        match self.tokens.get(self.current as usize) {
            Some(tk) => {
                self.current += 1;
                if tk.tk_type == tk_type {
                    tk
                } else {
                    bail!(ParseError::UnexpectedToken {
                        src: self.source.clone(),
                        span: tk.address.clone().span.into(),
                        unexpected: tk.value.clone(),
                        expected: tk_type
                    })
                }
            }
            None => bail!(ParseError::UnexpectedEof),
        }
    }

    /// Skips one token by adding 1 to current
    pub(crate) fn bump(&mut self) {
        self.current += 1;
    }

    /// Check current token type is equal to tk_type
    pub(crate) fn check(&self, tk_type: TokenKind) -> bool {
        match self.tokens.get(self.current as usize) {
            Some(tk) => tk.tk_type == tk_type,
            None => false,
        }
    }

    /// Check next token type is equal to tk_type
    pub(crate) fn check_next(&self, tk_type: TokenKind) -> bool {
        match self.tokens.get(self.current as usize + 1) {
            Some(tk) => tk.tk_type == tk_type,
            None => false,
        }
    }

    /// Peeks current token, if eof raises error
    pub(crate) fn peek(&self) -> &Token {
        match self.tokens.get(self.current as usize) {
            Some(tk) => tk,
            None => bail!(ParseError::UnexpectedEof),
        }
    }

    /// Peeks previous token, if eof raises error
    pub(crate) fn previous(&self) -> &Token {
        match self.tokens.get((self.current - 1) as usize) {
            Some(tk) => tk,
            None => bail!(ParseError::UnexpectedEof),
        }
    }

    /// Check `self.current >= self.tokens.len()`
    pub(crate) fn is_at_end(&self) -> bool {
        self.current as usize >= self.tokens.len()
    }
}
