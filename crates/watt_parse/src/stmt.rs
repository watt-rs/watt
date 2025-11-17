/// Imports
use crate::{errors::ParseError, parser::Parser};
use watt_ast::ast::{BinaryOp, Expression, Statement};
use watt_common::{address::Address, bail};
use watt_lex::tokens::TokenKind;

/// Implementation of statements parsing
impl<'file> Parser<'file> {
    /// Assignment parsing
    fn assignment(&mut self, address: Address, variable: Expression) -> Statement {
        match variable {
            Expression::Call { location, .. } => bail!(ParseError::InvalidAssignmentOperation {
                src: location.source,
                span: location.span.into()
            }),
            _ => {
                let op = self.advance().clone();
                match op.tk_type {
                    TokenKind::Assign => {
                        let start_address = variable.location();
                        let expr = self.expr();
                        let end_address = self.previous().address.clone();
                        Statement::VarAssign {
                            location: start_address + end_address,
                            what: variable,
                            value: expr,
                        }
                    }
                    TokenKind::AddAssign => {
                        let start_address = variable.location();
                        let expr = Box::new(self.expr());
                        let end_address = self.previous().address.clone();
                        Statement::VarAssign {
                            location: address.clone() + end_address.clone(),
                            what: variable.clone(),
                            value: Expression::Bin {
                                location: start_address + end_address,
                                left: Box::new(variable),
                                right: expr,
                                op: BinaryOp::Add,
                            },
                        }
                    }
                    TokenKind::SubAssign => {
                        let start_address = variable.location();
                        let expr = Box::new(self.expr());
                        let end_address = self.previous().address.clone();
                        Statement::VarAssign {
                            location: address + end_address.clone(),
                            what: variable.clone(),
                            value: Expression::Bin {
                                location: start_address + end_address,
                                left: Box::new(variable),
                                right: expr,
                                op: BinaryOp::Sub,
                            },
                        }
                    }
                    TokenKind::MulAssign => {
                        let start_address = variable.location();
                        let expr = Box::new(self.expr());
                        let end_address = self.previous().address.clone();
                        Statement::VarAssign {
                            location: address + end_address.clone(),
                            what: variable.clone(),
                            value: Expression::Bin {
                                location: start_address + end_address,
                                left: Box::new(variable),
                                right: expr,
                                op: BinaryOp::Mul,
                            },
                        }
                    }
                    TokenKind::DivAssign => {
                        let start_address = variable.location();
                        let expr = Box::new(self.expr());
                        let end_address = self.previous().address.clone();
                        Statement::VarAssign {
                            location: address + end_address.clone(),
                            what: variable.clone(),
                            value: Expression::Bin {
                                location: start_address + end_address,
                                left: Box::new(variable),
                                right: expr,
                                op: BinaryOp::Div,
                            },
                        }
                    }
                    _ => bail!(ParseError::InvalidAssignmentOperator {
                        src: address.source,
                        span: op.address.span.into(),
                        op: op.value
                    }),
                }
            }
        }
    }

    /// Let statement parsing
    fn let_stmt(&mut self) -> Statement {
        // start address
        let start_address = self.peek().address.clone();

        // `let $id`
        self.consume(TokenKind::Let);
        let name = self.consume(TokenKind::Id).clone();

        // if type specified
        let typ = if self.check(TokenKind::Colon) {
            // `: $type`
            self.consume(TokenKind::Colon);
            Option::Some(self.type_annotation())
        } else {
            // setting type to None
            Option::None
        };

        // `= $value`
        self.consume(TokenKind::Assign);
        let value = self.expr();

        // end address
        let end_address = self.previous().address.clone();

        Statement::VarDef {
            location: start_address + end_address,
            name: name.value,
            typ,
            value,
        }
    }

    /// Loop statement parsing
    fn loop_stmt(&mut self) -> Statement {
        let start_span = self.consume(TokenKind::Loop).address.clone();
        let logical = self.expr();
        let body = self.block_or_expr();
        let end_span = self.previous().address.clone();

        Statement::Loop {
            location: start_span + end_span,
            logical,
            body,
        }
    }

    /// For statement parsing
    fn for_stmt(&mut self) -> Statement {
        let start_span = self.consume(TokenKind::For).address.clone();
        let name = self.consume(TokenKind::Id).value.clone();
        self.consume(TokenKind::In);
        let range = Box::new(self.range());
        let body = self.block_or_expr();
        let end_span = self.previous().address.clone();

        Statement::For {
            location: start_span + end_span,
            name,
            range,
            body,
        }
    }

    /// Expression statement parsing
    fn expr_statement(&mut self) -> Statement {
        let expr = self.expr();
        if self.check(TokenKind::Semicolon) {
            Statement::Semi(expr)
        } else {
            Statement::Expr(expr)
        }
    }

    /// Statement parsing
    pub(crate) fn statement(&mut self) -> Statement {
        // Parsing statement
        let stmt = match self.peek().tk_type {
            TokenKind::Loop => self.loop_stmt(),
            TokenKind::For => self.for_stmt(),
            TokenKind::Let => self.let_stmt(),
            TokenKind::Id => {
                let pos = self.current;
                let start = self.peek().address.clone();
                let variable = self.variable();
                let end = self.peek().address.clone();
                match self.peek().tk_type {
                    TokenKind::AddAssign
                    | TokenKind::DivAssign
                    | TokenKind::MulAssign
                    | TokenKind::SubAssign
                    | TokenKind::Assign => self.assignment(start + end, variable),
                    _ => {
                        self.current = pos; // recovering to old position, if not an assignment
                        self.expr_statement()
                    }
                }
            }
            _ => self.expr_statement(),
        };
        // If `;` presented
        if self.check(TokenKind::Semicolon) {
            self.advance();
            stmt
        }
        // If not
        else {
            // Checking for closing brace `}`
            if self.check(TokenKind::Rbrace) {
                stmt
            } else {
                bail!(ParseError::ExpectedSemicolon {
                    src: self.source.clone(),
                    span: stmt.location().span.into()
                })
            }
        }
    }
}
