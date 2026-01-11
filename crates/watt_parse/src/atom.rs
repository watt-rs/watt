/// Imports
use crate::parser::Parser;
use ecow::EcoString;
use watt_ast::ast::{DependencyPath, Expression, Parameter, Range, TypePath};
use watt_lex::tokens::TokenKind;

/// Atom parse module
///
/// This implementation provides
/// some very little, but important
/// structures parsing.
///
impl<'file> Parser<'file> {
    /// List 'o items parsing `$open $item $sep $item $sep ...n $close`
    pub(crate) fn sep_by<T>(
        &mut self,
        open: TokenKind,
        close: TokenKind,
        sep: TokenKind,
        mut parse_item: impl FnMut(&mut Self) -> T,
    ) -> Vec<T> {
        let mut items = Vec::new();
        self.consume(open);

        if !self.check(close) {
            loop {
                items.push(parse_item(self));
                if self.check(sep) {
                    self.consume(sep);
                    if self.check(close) {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        self.consume(close);
        items
    }

    /// Arguments parsing `($expr, $expr, n...)`
    pub(crate) fn args(&mut self) -> Vec<Expression> {
        self.sep_by(
            TokenKind::Lparen,
            TokenKind::Rparen,
            TokenKind::Comma,
            |s| s.expr(),
        )
    }

    /// Depednecy path parsing
    pub(crate) fn dependency_path(&mut self) -> DependencyPath {
        // module name string
        let mut module = EcoString::new();
        // start address
        let start_address = self.peek().address.clone();

        // first `id`
        module.push_str(&self.consume(TokenKind::Id).value.clone());

        // while path separator exists, parsing new segment
        while self.check(TokenKind::Slash) {
            self.consume(TokenKind::Slash);
            module.push('/');
            module.push_str(&self.consume(TokenKind::Id).value.clone());
        }

        // end address
        let end_address = self.previous().address.clone();

        DependencyPath {
            address: start_address + end_address,
            module,
        }
    }

    /// Type annotation parsing
    pub(crate) fn type_annotation(&mut self) -> TypePath {
        // If function type annotation
        if self.check(TokenKind::Fn) {
            // start of span `fn (...): ...`
            let start_address = self.peek().address.clone();
            self.consume(TokenKind::Fn);
            // params
            let mut params: Vec<TypePath> = Vec::new();

            // `($type, $type, n )`
            self.consume(TokenKind::Lparen);
            if !self.check(TokenKind::Rparen) {
                params.push(self.type_annotation());

                while self.check(TokenKind::Comma) {
                    self.consume(TokenKind::Comma);
                    params.push(self.type_annotation());
                }
            }
            self.consume(TokenKind::Rparen);

            // : $ret
            let ret = if self.check(TokenKind::Colon) {
                self.consume(TokenKind::Colon);
                Some(Box::new(self.type_annotation()))
            } else {
                None
            };
            // end of span `fn (...): ...`
            let end_address = self.previous().address.clone();
            // function type path
            TypePath::Function {
                location: start_address + end_address,
                params,
                ret,
            }
        }
        // If unit type annotation
        else if self.check(TokenKind::Lparen) {
            // ()
            let start_address = self.advance().address.clone();
            let end_address = self.consume(TokenKind::Rparen).address.clone();
            TypePath::Unit {
                location: start_address + end_address,
            }
        }
        // Else, type name annotation
        else {
            // start address of `type.annotation`
            let start_address = self.peek().address.clone();
            // fisrt id
            let first_id = self.consume(TokenKind::Id).clone();
            // if dot found
            if self.check(TokenKind::Dot) {
                // consuming dot
                self.consume(TokenKind::Dot);
                // second id
                let second_id = self.consume(TokenKind::Id).clone();
                // generic
                let generics = if self.check(TokenKind::Lbracket) {
                    self.generic_args()
                } else {
                    Vec::new()
                };
                // end address of `module.definition`
                let end_address = self.previous().address.clone();
                // module type path
                TypePath::Module {
                    location: start_address + end_address,
                    module: first_id.value,
                    name: second_id.value,
                    generics,
                }
            }
            // else
            else {
                // generic
                let generics = if self.check(TokenKind::Lbracket) {
                    self.generic_args()
                } else {
                    Vec::new()
                };
                // end address of `module.definition`
                let end_address = self.previous().address.clone();
                // local type path
                TypePath::Local {
                    location: start_address + end_address,
                    name: first_id.value,
                    generics,
                }
            }
        }
    }

    /// Single parameter parsing
    pub(crate) fn parameter(&mut self) -> Parameter {
        // `$name: $typ`
        let name = self.consume(TokenKind::Id).clone();
        self.consume(TokenKind::Colon);
        let typ = self.type_annotation();

        Parameter {
            location: name.address,
            name: name.value,
            typ,
        }
    }

    /// Parameters parsing `($name: $type, $name: $type, ..n)`
    pub(crate) fn parameters(&mut self) -> Vec<Parameter> {
        self.sep_by(
            TokenKind::Lparen,
            TokenKind::Rparen,
            TokenKind::Comma,
            |s| s.parameter(),
        )
    }

    /// Generic parameters parsing `[$name, $name ..n]`
    pub(crate) fn generics(&mut self) -> Vec<EcoString> {
        self.sep_by(
            TokenKind::Lbracket,
            TokenKind::Rbracket,
            TokenKind::Comma,
            |s| s.consume(TokenKind::Id).value.clone(),
        )
    }

    /// Generic arguments parsing `[$type, $type ..n]`
    pub(crate) fn generic_args(&mut self) -> Vec<TypePath> {
        self.sep_by(
            TokenKind::Lbracket,
            TokenKind::Rbracket,
            TokenKind::Comma,
            |s| s.type_annotation(),
        )
    }

    /// Parses range
    ///
    /// # Example
    /// `0..3`
    /// `0..=10`
    /// `7..=280`
    ///
    pub(crate) fn range(&mut self) -> Range {
        // from..
        let from = self.expr();
        self.consume(TokenKind::Range);
        // Checking for `=`
        // If found => including last
        if self.check(TokenKind::Assign) {
            self.advance();
            let to = self.expr();
            Range::IncludeLast {
                location: from.location() + to.location(),
                from,
                to,
            }
        }
        // Else => excluding last
        else {
            let to = self.expr();
            Range::ExcludeLast {
                location: from.location() + to.location(),
                from,
                to,
            }
        }
    }
}
