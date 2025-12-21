/// Imports
use crate::{errors::ParseError, parser::Parser};
use watt_ast::ast::{
    ConstDeclaration, Declaration, Dependency, EnumConstructor, Field, FnDeclaration, Publicity,
    TypeDeclaration, UseKind,
};
use watt_common::bail;
use watt_lex::tokens::TokenKind;

/// Implementation of declarations parsing
impl<'file> Parser<'file> {
    /// Fn declaration parsing
    fn fn_declaration(&mut self, publicity: Publicity) -> FnDeclaration {
        // parsing function name
        let start_location = self.peek().address.clone();
        self.consume(TokenKind::Fn);
        let name = self.consume(TokenKind::Id).value.clone();

        // parsing function generics `[A, B, ...n]`
        let generics = if self.check(TokenKind::Lbracket) {
            self.generics()
        } else {
            Vec::new()
        };

        // parsing function parameters `(a: t1, b: t2, ...n)`
        let params = if self.check(TokenKind::Lparen) {
            self.parameters()
        } else {
            Vec::new()
        };

        // parsing return type, if given
        let typ = if self.check(TokenKind::Colon) {
            // `: $type`
            self.consume(TokenKind::Colon);
            Some(self.type_annotation())
        }
        // if type is not given, it will be inferred to unit later
        else {
            None
        };

        // parsing function body
        let body = self.block_or_expr();
        let end_location = self.previous().address.clone();

        FnDeclaration::Function {
            location: start_location + end_location,
            publicity,
            name,
            generics,
            params,
            body,
            typ,
        }
    }

    /// Constant declaration parsing
    fn const_declaration(&mut self, publicity: Publicity) -> ConstDeclaration {
        // parsing constant name `const $id`
        self.consume(TokenKind::Const);
        let name = self.consume(TokenKind::Id).clone();

        // parsing required type annotation `: $type`
        self.consume(TokenKind::Colon);
        let typ = self.type_annotation();

        // parsing constant value `= $value`
        self.consume(TokenKind::Assign);
        let value = self.expr();

        // checking give constant value is const
        self.check_value_const(&value);

        ConstDeclaration {
            location: name.address,
            publicity,
            name: name.value,
            typ,
            value,
        }
    }

    /// Extern fn declaration parsing
    fn extern_fn_declaration(&mut self, publicity: Publicity) -> FnDeclaration {
        // parsing function name
        let start_location = self.peek().address.clone();
        self.consume(TokenKind::Extern);
        self.consume(TokenKind::Fn);
        let name = self.consume(TokenKind::Id).value.clone();

        // parsing function generics `[A, B, ...n]`
        let generics = if self.check(TokenKind::Lbracket) {
            self.generics()
        } else {
            Vec::new()
        };

        // parsing function parameters `(a: t1, b: t2, ...n)`
        let params = if self.check(TokenKind::Lparen) {
            self.parameters()
        } else {
            Vec::new()
        };

        // parsing return type, if given
        let typ = if self.check(TokenKind::Colon) {
            // `: $type`
            self.consume(TokenKind::Colon);
            Some(self.type_annotation())
        }
        // if type is not given, it will be inferred to unit later
        else {
            None
        };

        // parsing function body
        self.consume(TokenKind::Assign);
        let body = self.consume(TokenKind::Text).value.clone();
        let end_location = self.previous().address.clone();

        FnDeclaration::ExternFunction {
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
        // parsing field name
        let start_location = self.peek().address.clone();
        let name = self.consume(TokenKind::Id).value.clone();

        // parsing required type annotation of field
        self.consume(TokenKind::Colon);
        let typ = self.type_annotation();
        let end_location = self.previous().address.clone();

        Field {
            location: start_location + end_location,
            name,
            typ,
        }
    }

    /// Type declaration parsing
    fn type_declaration(&mut self, publicity: Publicity) -> TypeDeclaration {
        // parsing type name
        let start_location = self.peek().address.clone();
        self.consume(TokenKind::Type);
        let name = self.consume(TokenKind::Id).clone();

        // parsing generic parameters
        let generics = if self.check(TokenKind::Lbracket) {
            self.generics()
        } else {
            Vec::new()
        };

        // parsing fields
        let fields = self.sep_by(
            TokenKind::Lbrace,
            TokenKind::Rbrace,
            TokenKind::Comma,
            |s| s.field(),
        );
        let end_location = self.previous().address.clone();

        TypeDeclaration::Struct {
            location: start_location + end_location,
            publicity,
            name: name.value,
            fields,
            generics,
        }
    }

    /// Enum variant parsing
    fn variant(&mut self) -> EnumConstructor {
        // variant name
        let start_location = self.peek().address.clone();
        let name = self.consume(TokenKind::Id).value.clone();

        // variant param
        let params = if self.check(TokenKind::Lparen) {
            self.parameters()
        } else {
            Vec::new()
        };
        let end_location = self.peek().address.clone();

        EnumConstructor {
            location: start_location + end_location,
            name,
            params,
        }
    }

    /// Enum declaration parsing
    fn enum_declaration(&mut self, publicity: Publicity) -> TypeDeclaration {
        // parsing enum name
        let start_location = self.peek().address.clone();
        self.consume(TokenKind::Enum);
        let name = self.consume(TokenKind::Id).clone();

        // parsing enum generic
        let generics = if self.check(TokenKind::Lbracket) {
            self.generics()
        } else {
            Vec::new()
        };
        let end_location = self.previous().address.clone();

        // variants parsing
        let variants = self.sep_by(
            TokenKind::Lbrace,
            TokenKind::Rbrace,
            TokenKind::Comma,
            |s| s.variant(),
        );

        TypeDeclaration::Enum {
            location: start_location + end_location,
            publicity,
            name: name.value,
            generics,
            variants,
        }
    }

    /// Use declaration `use ...` | `use (..., ..., n)` parsing
    pub(crate) fn use_declaration(&mut self) -> Dependency {
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
    pub(crate) fn declaration(&mut self, publicity: Publicity) -> Declaration {
        match self.peek().tk_type {
            TokenKind::Type => Declaration::Type(self.type_declaration(publicity)),
            TokenKind::Fn => Declaration::Fn(self.fn_declaration(publicity)),
            TokenKind::Enum => Declaration::Type(self.enum_declaration(publicity)),
            TokenKind::Const => Declaration::Const(self.const_declaration(publicity)),
            TokenKind::Extern => Declaration::Fn(self.extern_fn_declaration(publicity)),
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
}
