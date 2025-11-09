/// Imports
use crate::errors::ParseError;
use miette::NamedSource;
use std::sync::Arc;
use watt_ast::ast::*;
use watt_common::bail;
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
        let start_span = self.peek().address.clone();
        self.consume(TokenKind::Lbrace);
        while !self.check(TokenKind::Rbrace) {
            nodes.push(self.statement());
        }
        self.consume(TokenKind::Rbrace);
        let end_span = self.previous().address.clone();

        Block {
            location: start_span + end_span,
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
            let start_span = self.peek().address.clone();
            while !self.check(TokenKind::Rbrace) {
                nodes.push(self.statement());
            }
            let end_span = self.previous().address.clone();
            self.consume(TokenKind::Rbrace);
            Either::Left(Block {
                location: start_span + end_span,
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
            let start_span = self.peek().address.clone();
            while !self.check(TokenKind::Rbrace) {
                nodes.push(self.statement());
            }
            let end_span = self.previous().address.clone();
            self.consume(TokenKind::Rbrace);
            Either::Left(Block {
                location: start_span + end_span,
                body: nodes,
            })
        } else {
            // `=`
            self.consume(TokenKind::Assign);
            // parsing single expression
            Either::Right(Box::new(self.expr()))
        }
    }

    /// Fn declaration parsing
    fn fn_declaration(&mut self, publicity: Publicity) -> Declaration {
        // start location
        let start_location = self.peek().address.clone();
        self.consume(TokenKind::Fn);

        // function name
        let name = self.consume(TokenKind::Id).value.clone();

        // generic
        let generics = if self.check(TokenKind::Lbracket) {
            self.generics()
        } else {
            Vec::new()
        };

        // params
        let params = if self.check(TokenKind::Lparen) {
            self.parameters()
        } else {
            Vec::new()
        };

        // return type
        // if type specified
        let typ = if self.check(TokenKind::Colon) {
            // `: $type`
            self.consume(TokenKind::Colon);
            Some(self.type_annotation())
        }
        // else
        else {
            None
        };

        // body
        let body = self.block_or_expr();

        // end location
        let end_location = self.previous().address.clone();

        Declaration::Function {
            location: start_location + end_location,
            publicity,
            name,
            generics,
            params,
            body,
            typ,
        }
    }

    /// Let declaration parsing
    fn let_declaration(&mut self, publicity: Publicity) -> Declaration {
        // `let $id`
        self.consume(TokenKind::Let);
        let name = self.consume(TokenKind::Id).clone();

        // if type specified
        let typ = if self.check(TokenKind::Colon) {
            // `: $type`
            self.consume(TokenKind::Colon);
            Option::Some(self.type_annotation())
        }
        // else
        else {
            // setting type to None
            Option::None
        };

        // `= $value`
        self.consume(TokenKind::Assign);
        let value = self.expr();

        Declaration::VarDef {
            location: name.address,
            publicity,
            name: name.value,
            typ,
            value,
        }
    }

    /// Extern fn declaration parsing
    fn extern_fn_declaration(&mut self, publicity: Publicity) -> Declaration {
        // start location
        let start_location = self.peek().address.clone();

        self.consume(TokenKind::Extern);
        self.consume(TokenKind::Fn);

        // function name
        let name = self.consume(TokenKind::Id).value.clone();

        // generic
        let generics = if self.check(TokenKind::Lbracket) {
            self.generics()
        } else {
            Vec::new()
        };

        // params
        let params = if self.check(TokenKind::Lparen) {
            self.parameters()
        } else {
            Vec::new()
        };

        // return type
        // if type specified
        let typ = if self.check(TokenKind::Colon) {
            // `: $type`
            self.consume(TokenKind::Colon);
            Some(self.type_annotation())
        }
        // else
        else {
            None
        };

        // body
        self.consume(TokenKind::Assign);
        let body = self.consume(TokenKind::Text).value.clone();

        // end location
        let end_location = self.previous().address.clone();

        Declaration::ExternFunction {
            location: start_location + end_location,
            name,
            publicity,
            generics,
            params,
            typ,
            body,
        }
    }

    /// Type field
    fn field(&mut self) -> Field {
        // start address
        let start_address = self.peek().address.clone();

        // field name
        let name = self.consume(TokenKind::Id).value.clone();

        // type annotation
        self.consume(TokenKind::Colon);
        let typ = self.type_annotation();

        // end address
        let end_address = self.previous().address.clone();

        Field {
            location: start_address + end_address,
            name,
            typ,
        }
    }

    /// Type declaration parsing
    fn type_declaration(&mut self, publicity: Publicity) -> Declaration {
        // start address
        let start_address = self.peek().address.clone();

        // variable is used to create type span in bails.
        self.consume(TokenKind::Type);

        // type name
        let name = self.consume(TokenKind::Id).clone();

        // generic
        let generics = if self.check(TokenKind::Lbracket) {
            self.generics()
        } else {
            Vec::new()
        };

        // type contents
        let mut fields = Vec::new();

        // body parsing
        if self.check(TokenKind::Lbrace) {
            self.consume(TokenKind::Lbrace);
            while !self.check(TokenKind::Rbrace) {
                fields.push(self.field())
            }
            self.consume(TokenKind::Rbrace);
        }

        // end address
        let end_address = self.previous().address.clone();

        Declaration::TypeDeclaration {
            location: start_address + end_address,
            publicity,
            name: name.value,
            fields,
            generics,
        }
    }

    /// Enum declaration parsing
    fn enum_declaration(&mut self, publicity: Publicity) -> Declaration {
        // start address
        let start_location = self.peek().address.clone();

        // variable is used to create type span in bails.
        self.consume(TokenKind::Enum);

        // type name
        let name = self.consume(TokenKind::Id).clone();

        // generic
        let generics = if self.check(TokenKind::Lbracket) {
            self.generics()
        } else {
            Vec::new()
        };

        // creating type address
        let end_location = self.previous().address.clone();

        // type contents
        let mut variants = Vec::new();

        // variants parsing
        if self.check(TokenKind::Lbrace) {
            self.consume(TokenKind::Lbrace);
            while !self.check(TokenKind::Rbrace) {
                // start address
                let start_location = self.peek().address.clone();
                let name = self.consume(TokenKind::Id).value.clone();
                let params = if self.check(TokenKind::Lparen) {
                    self.parameters()
                } else {
                    Vec::new()
                };
                // end address
                let end_location = self.peek().address.clone();
                variants.push(EnumConstructor {
                    location: start_location + end_location,
                    name,
                    params,
                });
                if self.check(TokenKind::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
            self.consume(TokenKind::Rbrace);
        }

        Declaration::EnumDeclaration {
            location: start_location + end_location,
            publicity,
            name: name.value,
            generics,
            variants,
        }
    }

    /// Use declaration `use ...` | `use (..., ..., n)` parsing
    fn use_declaration(&mut self) -> Dependency {
        // start of span `use ... as ...`
        let start_location = self.peek().address.clone();

        // `use` keyword
        self.consume(TokenKind::Use);

        // `path/to/module`
        let path = self.dependency_path();

        // `for $name, $name, n...`
        let kind = if self.check(TokenKind::For) {
            self.consume(TokenKind::For);
            // Parsing names
            let mut names = Vec::new();
            names.push(self.consume(TokenKind::Id).clone());
            while self.check(TokenKind::Comma) {
                self.advance();
                names.push(self.consume(TokenKind::Id).clone());
            }
            UseKind::ForNames(names.into_iter().map(|tk| tk.value).collect())
        }
        // `as $id`
        else {
            self.consume(TokenKind::As);
            let as_name = self.consume(TokenKind::Id).clone();
            UseKind::AsName(as_name.value)
        };

        // end of span `use ... as ...`
        let end_location = self.previous().address.clone();

        Dependency {
            location: start_location + end_location,
            path,
            kind,
        }
    }

    /// Declaration parsing
    fn declaration(&mut self, publicity: Publicity) -> Declaration {
        match self.peek().tk_type {
            TokenKind::Type => self.type_declaration(publicity),
            TokenKind::Fn => self.fn_declaration(publicity),
            TokenKind::Enum => self.enum_declaration(publicity),
            TokenKind::Let => self.let_declaration(publicity),
            TokenKind::Extern => self.extern_fn_declaration(publicity),
            _ => {
                let token = self.peek().clone();
                bail!(ParseError::UnexpectedDeclarationToken {
                    src: token.address.source,
                    span: token.address.span.into(),
                    unexpected: token.value
                })
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
