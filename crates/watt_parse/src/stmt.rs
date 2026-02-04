/// Imports
use crate::{errors::ParseError, parser::Parser};
use watt_ast::ast::{BinaryOp, Expression, Statement};
use watt_common::{address::Address, bail};
use watt_lex::tokens::TokenKind;

/// Implementation of statements parsing
impl<'file> Parser<'file> {
    /// Packs compound assignment
    fn compound_assignment(
        &mut self,
        op: BinaryOp,
        address: Address,
        variable: Expression,
    ) -> Statement {
        let span_start = variable.location();
        let expr = Box::new(self.expr());
        let span_end = self.previous().address.clone();
        Statement::VarAssign {
            location: address + span_end.clone(),
            what: variable.clone(),
            value: Expression::Bin {
                location: span_start + span_end,
                left: Box::new(variable),
                right: expr,
                op,
            },
        }
    }

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
                        let span_start = variable.location();
                        let expr = self.expr();
                        let span_end = self.previous().address.clone();
                        Statement::VarAssign {
                            location: span_start + span_end,
                            what: variable,
                            value: expr,
                        }
                    }
                    TokenKind::AddAssign => {
                        self.compound_assignment(BinaryOp::Add, address, variable)
                    }
                    TokenKind::SubAssign => {
                        self.compound_assignment(BinaryOp::Sub, address, variable)
                    }
                    TokenKind::MulAssign => {
                        self.compound_assignment(BinaryOp::Mul, address, variable)
                    }
                    TokenKind::DivAssign => {
                        self.compound_assignment(BinaryOp::Div, address, variable)
                    }
                    TokenKind::AndAssign => {
                        self.compound_assignment(BinaryOp::And, address, variable)
                    }
                    TokenKind::OrAssign => {
                        self.compound_assignment(BinaryOp::Or, address, variable)
                    }
                    TokenKind::XorAssign => {
                        self.compound_assignment(BinaryOp::Xor, address, variable)
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
        // `let $id`
        let span_start = self.consume(TokenKind::Let).address.clone();
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
        let span_end = self.previous().address.clone();

        Statement::VarDef {
            location: span_start + span_end,
            name: name.value,
            typ,
            value,
        }
    }

    /// Loop statement parsing
    fn loop_stmt(&mut self) -> Statement {
        let start_location = self.consume(TokenKind::Loop).address.clone();
        let logical = self.expr();
        let body = self.block_or_expr();
        let end_location = self.previous().address.clone();

        Statement::Loop {
            location: start_location + end_location,
            logical,
            body,
        }
    }

    /// For statement parsing
    fn for_stmt(&mut self) -> Statement {
        let start_location = self.consume(TokenKind::For).address.clone();
        let name = self.consume(TokenKind::Id).value.clone();
        self.consume(TokenKind::In);
        let range = Box::new(self.range());

        // body parsing
        let body = self.block_or_expr();
        let end_location = self.previous().address.clone();

        Statement::For {
            location: start_location + end_location,
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

    /// Is statement requires semicolon
    fn statement_requires_semi(&self, stmt: &Statement) -> bool {
        match stmt {
            Statement::Loop { .. } => false,
            Statement::For { .. } => false,
            Statement::Expr(Expression::If { .. }) => false,
            _ => true,
        }
    }

    /// Identifier statement
    fn id_stmt(&mut self) -> Statement {
        // point for the recover
        let recover_point = self.current;
        let start = self.peek().address.clone();

        // parsing variable
        let variable = self.variable();
        let end = self.peek().address.clone();

        // checking for assignment operators
        match self.peek().tk_type {
            // if found, parsing assignment
            TokenKind::AddAssign
            | TokenKind::DivAssign
            | TokenKind::MulAssign
            | TokenKind::SubAssign
            | TokenKind::Assign => self.assignment(start + end, variable),
            // if not, recovering to `recovert_point` and parsing expr-statement
            _ => {
                self.current = recover_point;
                self.expr_statement()
            }
        }
    }

    /// Statement parsing
    pub(crate) fn statement(&mut self) -> Statement {
        // parsing statement
        let stmt = match self.peek().tk_type {
            TokenKind::Loop => self.loop_stmt(),
            TokenKind::For => self.for_stmt(),
            TokenKind::Let => self.let_stmt(),
            TokenKind::Id => self.id_stmt(),
            _ => self.expr_statement(),
        };
        // if `;` presented
        if self.check(TokenKind::Semicolon) {
            self.advance();
            stmt
        }
        // if not
        else {
            // if here's closing brace of the block, or the statement
            // does not need a semicolon, just returning it.
            if self.check(TokenKind::Rbrace) | !self.statement_requires_semi(&stmt) {
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
