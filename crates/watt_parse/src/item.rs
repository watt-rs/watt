/// Imports
use crate::{Parser, errors::ParseError};
use watt_ast::{
    atom::Publicity,
    item::{Enum, Field, Fn, Import, ImportKind, ImportPath, Item, Struct, Variant},
};
use watt_lex::token::TokenKind;
use watt_macros::bail;

/// Item parsing implementation
impl<'s> Parser<'s> {
    // Parses struct field
    fn struct_field(&mut self) -> Field {
        let start_span = self.peek().span.clone();
        let name = self.expect(TokenKind::Id).lexeme;
        self.expect(TokenKind::Colon);
        let hint = self.type_hint();
        let end_span = self.prev().span.clone();

        Field {
            span: start_span + end_span,
            name,
            hint,
        }
    }

    // Parses struct
    fn struct_item(&mut self, publicity: Publicity) -> Item {
        // Bumping `struct`
        let start_span = self.peek().span.clone();
        self.bump();

        // Parsing signature
        let name = self.expect(TokenKind::Id).lexeme;
        let generics = self.generic_params();

        // Parsing fields
        let fields = self.sep_by(
            TokenKind::Lbrace,
            TokenKind::Rbrace,
            TokenKind::Comma,
            |p| p.struct_field(),
        );
        let end_span = self.prev().span.clone();

        Item::Struct(Struct {
            span: start_span + end_span,
            publicity,
            name,
            generics,
            fields,
        })
    }

    // Parses enum variant
    fn enum_variant(&mut self) -> Variant {
        // Parsing enum variant
        let start_span = self.peek().span.clone();
        let name = self.expect(TokenKind::Id).lexeme;
        let params = if self.check(TokenKind::Lparen) {
            self.sep_by(
                TokenKind::Lparen,
                TokenKind::Rparen,
                TokenKind::Comma,
                |p| p.type_hint(),
            )
        } else {
            Vec::new()
        };
        let end_span = self.prev().span.clone();

        Variant {
            span: start_span + end_span,
            name,
            fields: params,
        }
    }

    // Parses enum
    fn enum_item(&mut self, publicity: Publicity) -> Item {
        // Bumping `enum`
        let start_span = self.peek().span.clone();
        self.bump();

        // Parsing signature
        let name = self.expect(TokenKind::Id).lexeme;
        let generics = self.generic_params();

        // Parsing variants
        let variants = self.sep_by(
            TokenKind::Lbrace,
            TokenKind::Rbrace,
            TokenKind::Comma,
            |p| p.enum_variant(),
        );
        let end_span = self.prev().span.clone();

        Item::Enum(Enum {
            span: start_span + end_span,
            publicity,
            name,
            generics,
            variants,
        })
    }

    // Parses function
    fn fn_item(&mut self, publicity: Publicity) -> Item {
        // Bumping `fn`
        let start_span = self.peek().span.clone();
        self.bump();

        // Parsing signature
        let name = self.expect(TokenKind::Id).lexeme;
        let generics = self.generic_params();
        let params = self.params();
        let ret = if self.check(TokenKind::Colon) {
            self.bump();
            Some(self.type_hint())
        } else {
            None
        };

        // Parsing body
        let block = self.block();
        let end_span = self.prev().span.clone();

        Item::Fn(Fn {
            span: start_span + end_span,
            publicity,
            name,
            generics,
            params,
            ret,
            block,
        })
    }

    /// Import path parsing
    fn import_path(&mut self) -> ImportPath {
        // Module name string
        let start_span = self.peek().span.clone();
        let mut module = String::new();

        // First id
        module.push_str(&self.expect(TokenKind::Id).lexeme);

        while self.check(TokenKind::Slash) {
            self.bump();
            module.push('/');
            module.push_str(&self.expect(TokenKind::Id).lexeme);
        }
        let end_span = self.prev().span.clone();

        ImportPath {
            span: start_span + end_span,
            module,
        }
    }

    // Parses import item
    pub(crate) fn import(&mut self) -> Import {
        // Bumping `import`
        let start_span = self.peek().span.clone();
        self.bump();

        // Import path
        let path = self.import_path();

        // Suffix
        let kind = if self.check(TokenKind::As) {
            self.bump();
            let name = self.expect(TokenKind::Id).lexeme;

            ImportKind::As(name)
        } else if self.check(TokenKind::For) {
            self.bump();
            let names = self.sep_by_2(TokenKind::Comma, |p| p.expect(TokenKind::Id).lexeme);

            ImportKind::For(names)
        } else {
            ImportKind::Just
        };
        let end_span = self.prev().span.clone();

        Import {
            span: start_span + end_span,
            path,
            kind,
        }
    }

    // Parses top-level item
    pub(crate) fn item(&mut self, publicity: Publicity) -> Item {
        let tk = self.peek().clone();

        match &tk.kind {
            TokenKind::Struct => self.struct_item(publicity),
            TokenKind::Enum => self.enum_item(publicity),
            TokenKind::Fn => self.fn_item(publicity),
            _ => bail!(ParseError::UnexpectedItemToken {
                got: tk.kind,
                src: self.source.clone(),
                span: tk.span.1.into(),
            }),
        }
    }
}
