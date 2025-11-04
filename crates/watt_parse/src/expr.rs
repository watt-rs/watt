/// Imports
use crate::{errors::ParseError, parser::Parser};
use watt_ast::ast::{BinaryOp, Case, Either, ElseBranch, Expression, Parameter, Pattern, UnaryOp};
use watt_common::bail;
use watt_lex::tokens::TokenKind;

/// Implementation of epxression parsing
impl<'file> Parser<'file> {
    /// Anonymous fn expr
    fn anonymous_fn_expr(&mut self) -> Expression {
        // start span `fn (): ... {}`
        let start_span = self.peek().address.clone();
        self.consume(TokenKind::Fn);

        // params
        let mut params: Vec<Parameter> = Vec::new();
        if self.check(TokenKind::Lparen) {
            params = self.parameters();
        }

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
        // end span `fn (): ... {}`
        let end_span = self.previous().address.clone();

        // body
        let body = self.block_or_box_expr();

        Expression::Function {
            location: start_span + end_span,
            params,
            body,
            typ,
        }
    }

    /// Else parsing
    fn else_branch(&mut self) -> ElseBranch {
        let start_span = self.consume(TokenKind::Else).address.clone();
        let body = self.block_or_expr();
        let end_span = self.previous().address.clone();

        ElseBranch::Else {
            location: start_span + end_span,
            body,
        }
    }

    /// Elif parsing
    fn elif_branch(&mut self) -> ElseBranch {
        let start_span = self.consume(TokenKind::Elif).address.clone();
        let logical = self.expr();
        let body = self.block_or_expr();
        let end_span = self.previous().address.clone();

        ElseBranch::Elif {
            location: start_span + end_span,
            logical,
            body,
        }
    }

    /// If expression parsing
    fn if_expr(&mut self) -> Expression {
        let start_span = self.consume(TokenKind::If).address.clone();
        let logical = self.expr();
        let body = self.block_or_box_expr();
        let end_span = self.previous().address.clone();
        let mut else_branches = Vec::new();

        // elif parsing
        while self.check(TokenKind::Elif) {
            else_branches.push(self.elif_branch());
        }

        // else parsing
        if self.check(TokenKind::Else) {
            else_branches.push(self.else_branch());
        }

        Expression::If {
            location: start_span + end_span,
            logical: Box::new(logical),
            body,
            else_branches,
        }
    }

    /// Variable parsing
    pub(crate) fn variable(&mut self) -> Expression {
        // start address `id...`
        let start_address = self.peek().address.clone();

        // variable
        let variable = self.consume(TokenKind::Id).clone();

        // result node
        let mut result = Expression::PrefixVar {
            location: variable.address,
            name: variable.value,
        };

        // checking for dots and parens
        loop {
            // checking for chain `a.b.c.d`
            if self.check(TokenKind::Dot) {
                self.consume(TokenKind::Dot);
                let variable = self.consume(TokenKind::Id).clone();
                result = Expression::SuffixVar {
                    location: variable.address,
                    container: Box::new(result),
                    name: variable.value,
                };
                continue;
            }
            // checking for call
            if self.check(TokenKind::Lparen) {
                let args = self.args();
                let end_address = self.previous().address.clone();
                result = Expression::Call {
                    location: start_address.clone() + end_address,
                    what: Box::new(result),
                    args,
                };
                continue;
            }
            // breaking cycle
            break;
        }
        result
    }

    /// Grouping expr `( expr )`
    fn grouping_expr(&mut self) -> Expression {
        // `($expr)`
        self.consume(TokenKind::Lparen);
        let expr = self.expr();
        self.consume(TokenKind::Rparen);

        expr
    }

    /// Todo expr `todo`
    #[inline]
    fn todo_expr(&mut self) -> Expression {
        Expression::Todo {
            location: self.advance().address.clone(),
        }
    }

    /// Primary expr parsing
    fn primary_expr(&mut self) -> Expression {
        match self.peek().tk_type {
            TokenKind::Id => self.variable(),
            TokenKind::Number => {
                let value = self.advance().clone();
                if value.value.contains(".") {
                    Expression::Float {
                        location: value.address,
                        value: value.value,
                    }
                } else {
                    Expression::Int {
                        location: value.address,
                        value: value.value,
                    }
                }
            }
            TokenKind::Text => {
                let value = self.advance().clone();
                Expression::String {
                    location: value.address,
                    value: value.value,
                }
            }
            TokenKind::Bool => {
                let value = self.advance().clone();
                Expression::Bool {
                    location: value.address,
                    value: value.value,
                }
            }
            TokenKind::Todo => self.todo_expr(),
            TokenKind::Lparen => self.grouping_expr(),
            TokenKind::Fn => self.anonymous_fn_expr(),
            TokenKind::Match => self.pattern_matching(),
            TokenKind::If => self.if_expr(),
            _ => {
                let token = self.peek().clone();
                bail!(ParseError::UnexpectedExpressionToken {
                    src: token.address.source,
                    span: token.address.span.into(),
                    unexpected: token.value
                });
            }
        }
    }

    /// Unary expr `!` and `-` parsing
    fn unary_expr(&mut self) -> Expression {
        if self.check(TokenKind::Bang) || self.check(TokenKind::Minus) {
            let op = self.advance().clone();

            Expression::Unary {
                location: op.address,
                op: match op.tk_type {
                    TokenKind::Minus => UnaryOp::Neg,
                    TokenKind::Bang => UnaryOp::Bang,
                    _ => unreachable!(),
                },
                value: Box::new(self.primary_expr()),
            }
        } else {
            self.primary_expr()
        }
    }

    /// Binary operations `*`, `/`, `%`, `^`, `&`, `|` parsing
    fn multiplicative_expr(&mut self) -> Expression {
        let mut start_location = self.peek().address.clone();
        let mut left = self.unary_expr();

        while self.check(TokenKind::Star)
            || self.check(TokenKind::Slash)
            || self.check(TokenKind::Percent)
            || self.check(TokenKind::Caret)
            || self.check(TokenKind::Ampersand)
            || self.check(TokenKind::Bar)
        {
            let op = self.peek().clone();
            self.bump();
            let right = self.unary_expr();
            let end_location = self.previous().address.clone();
            left = Expression::Bin {
                location: start_location + end_location,
                left: Box::new(left),
                right: Box::new(right),
                op: match op.tk_type {
                    TokenKind::Star => BinaryOp::Mul,
                    TokenKind::Slash => BinaryOp::Div,
                    TokenKind::Ampersand => BinaryOp::BitwiseAnd,
                    TokenKind::Bar => BinaryOp::BitwiseOr,
                    TokenKind::Percent => BinaryOp::Mod,
                    _ => unreachable!(),
                },
            };
            start_location = self.peek().address.clone();
        }

        left
    }

    /// Binary operations `+`, `-`, '<>' parsing
    fn additive_expr(&mut self) -> Expression {
        let mut start_location = self.peek().address.clone();
        let mut left = self.multiplicative_expr();

        while self.check(TokenKind::Plus)
            || self.check(TokenKind::Minus)
            || self.check(TokenKind::Concat)
        {
            let op = self.peek().clone();
            self.bump();
            let right = self.multiplicative_expr();
            let end_location = self.previous().address.clone();
            left = Expression::Bin {
                location: start_location + end_location,
                left: Box::new(left),
                right: Box::new(right),
                op: match op.tk_type {
                    TokenKind::Plus => BinaryOp::Add,
                    TokenKind::Minus => BinaryOp::Sub,
                    TokenKind::Concat => BinaryOp::Concat,
                    _ => unreachable!(),
                },
            };
            start_location = self.peek().address.clone();
        }

        left
    }

    /// Compare operations `<`, `>`, `<=`, `>=` parsing
    fn compare_expr(&mut self) -> Expression {
        let start_location = self.peek().address.clone();
        let mut left = self.additive_expr();

        if self.check(TokenKind::Greater)
            || self.check(TokenKind::GreaterEq)
            || self.check(TokenKind::Less)
            || self.check(TokenKind::LessEq)
        {
            let op = self.advance().clone();
            let right = self.additive_expr();
            let end_location = self.previous().address.clone();
            left = Expression::Bin {
                location: start_location + end_location,
                left: Box::new(left),
                right: Box::new(right),
                op: match op.tk_type {
                    TokenKind::Greater => BinaryOp::Gt,
                    TokenKind::GreaterEq => BinaryOp::Ge,
                    TokenKind::Less => BinaryOp::Lt,
                    TokenKind::LessEq => BinaryOp::Le,
                    _ => unreachable!(),
                },
            };
        }

        left
    }

    /// Equality operations `==`, `!=` parsing
    fn equality_expr(&mut self) -> Expression {
        let start_location = self.peek().address.clone();
        let mut left = self.compare_expr();

        if self.check(TokenKind::Eq) || self.check(TokenKind::NotEq) {
            let op = self.advance().clone();
            let right = self.compare_expr();
            let end_location = self.previous().address.clone();
            left = Expression::Bin {
                location: start_location + end_location,
                left: Box::new(left),
                right: Box::new(right),
                op: match op.tk_type {
                    TokenKind::Eq => BinaryOp::Eq,
                    TokenKind::NotEq => BinaryOp::NotEq,
                    _ => unreachable!(),
                },
            };
        }

        left
    }

    /// Logical operations `and`, `or` parsing
    fn logical_expr(&mut self) -> Expression {
        let mut start_location = self.peek().address.clone();
        let mut left = self.equality_expr();

        while self.check(TokenKind::And) || self.check(TokenKind::Or) {
            let op = self.advance().clone();
            let right = self.equality_expr();
            let end_location = self.previous().address.clone();
            left = Expression::Bin {
                location: start_location + end_location,
                left: Box::new(left),
                right: Box::new(right),
                op: match op.tk_type {
                    TokenKind::And => BinaryOp::And,
                    TokenKind::Or => BinaryOp::Or,
                    _ => unreachable!(),
                },
            };
            start_location = self.peek().address.clone();
        }

        left
    }

    /// Variant pattern prefix.
    /// Example: `Option.Some`
    fn variant_pattern_prefix(&mut self) -> Expression {
        // variable
        let variable = self.consume(TokenKind::Id).clone();

        // result node
        let mut result = Expression::PrefixVar {
            location: variable.address,
            name: variable.value,
        };

        // checking for dots and parens
        loop {
            // checking for chain `a.b.c.d`
            if self.check(TokenKind::Dot) {
                self.consume(TokenKind::Dot);
                let variable = self.consume(TokenKind::Id).clone();
                result = Expression::SuffixVar {
                    location: variable.address,
                    container: Box::new(result),
                    name: variable.value,
                };
                continue;
            }
            // breaking cycle
            break;
        }
        result
    }

    /// Pattern parsing
    fn pattern(&mut self) -> Pattern {
        // Parsing single pattern
        let pattern =
            // If string presented
            if self.check(TokenKind::Text) {
                Pattern::String(self.advance().value.clone())
            }
            // If bool presented
            else if self.check(TokenKind::Bool) {
                Pattern::Bool(self.advance().value.clone())
            }
            // If number presented
            else if self.check(TokenKind::Number) {
                let value = self.advance().clone();
                if value.value.contains(".") {
                    Pattern::Float(value.value)
                } else {
                    Pattern::Int(value.value)
                }
            }
            // If wildcard presented
            else if self.check(TokenKind::Wildcard) {
                self.advance();
                Pattern::Wildcard
            }
            // If identifier presented
            else {
                // If dot presented -> enum patterns
                if self.check_next(TokenKind::Dot) {
                    // Parsing variant pattern prefix
                    let value = self.variant_pattern_prefix();
                    // Checking for unwrap of enum
                    if self.check(TokenKind::Lparen) {
                        // (.., n fields)
                        self.consume(TokenKind::Lparen);
                        let mut fields = Vec::new();
                        // Checking for close of parens
                        if self.check(TokenKind::Rparen) {
                            self.advance();
                            return Pattern::Unwrap { en: value, fields };
                        }
                        // Parsing field names
                        let field = self.consume(TokenKind::Id).clone();
                        fields.push((field.address, field.value));
                        while self.check(TokenKind::Comma) {
                            self.advance();
                            let field = self.consume(TokenKind::Id).clone();
                            fields.push((field.address, field.value));
                        }
                        self.consume(TokenKind::Rparen);
                        // As result, enum unwrap pattern
                        Pattern::Unwrap { en: value, fields }
                    }
                    // If no unwrap, returning just as value
                    else {
                        Pattern::Variant(value)
                    }
                }
                // If not -> bind pattern
                else {
                    Pattern::BindTo(self.consume(TokenKind::Id).value.clone())
                }
            };
        // Checking if more patterns presented
        if self.check(TokenKind::Bar) {
            // Parsing `or` pattern
            self.consume(TokenKind::Bar);
            Pattern::Or(Box::new(pattern), Box::new(self.pattern()))
        } else {
            pattern
        }
    }

    /// Pattern match parsing
    fn pattern_matching(&mut self) -> Expression {
        // Start address
        let start_address = self.peek().address.clone();

        // `match value { patterns, ... }`
        self.consume(TokenKind::Match);
        let value = self.expr();

        // Cases
        self.consume(TokenKind::Lbrace);
        let mut cases = Vec::new();
        while !self.check(TokenKind::Rbrace) {
            // Start address of case
            let start_span = self.peek().address.clone();
            // Pattern of case
            let pattern = self.pattern();
            // -> { body, ... }
            self.consume(TokenKind::Arrow);
            let body = if self.check(TokenKind::Lbrace) {
                Either::Left(self.block())
            } else {
                Either::Right(self.expr())
            };
            // End address of case
            let end_span = self.previous().address.clone();
            cases.push(Case {
                address: start_span + end_span,
                pattern,
                body,
            });
        }
        self.consume(TokenKind::Rbrace);

        // End address
        let end_address = self.previous().address.clone();

        Expression::Match {
            location: start_address + end_address,
            value: Box::new(value),
            cases,
        }
    }

    /// Expr parsing
    pub(crate) fn expr(&mut self) -> Expression {
        self.logical_expr()
    }
}
