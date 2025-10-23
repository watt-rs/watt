/// Imports
use crate::errors::ParseError;
use ecow::EcoString;
use miette::NamedSource;
use std::sync::Arc;
use watt_ast::ast::*;
use watt_common::address::Address;
use watt_common::bail;
use watt_lex::tokens::{Token, TokenKind};

/// Parser structure
pub struct Parser<'file_path> {
    tokens: Vec<Token>,
    current: u128,
    named_source: &'file_path NamedSource<Arc<String>>,
}

/// Parser implementation
#[allow(unused_qualifications)]
impl<'file_path> Parser<'file_path> {
    /// New parser
    pub fn new(tokens: Vec<Token>, named_source: &'file_path NamedSource<Arc<String>>) -> Self {
        Parser {
            tokens,
            current: 0,
            named_source,
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
            source: self.named_source.to_owned(),
            dependencies,
            declarations,
        }
    }

    /// Block statement parsing
    fn block(&mut self) -> Block {
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

        Block {
            location: start_span + end_span,
            body: nodes,
        }
    }

    /// Arguments parsing `($expr, $expr, n...)`
    fn args(&mut self) -> Vec<Expression> {
        // result list
        let mut nodes: Vec<Expression> = Vec::new();

        // `(Expression, Expression, n...)`
        self.consume(TokenKind::Lparen);
        if !self.check(TokenKind::Rparen) {
            nodes.push(self.expr());
            while self.check(TokenKind::Comma) {
                self.consume(TokenKind::Comma);
                nodes.push(self.expr());
            }
        }
        self.consume(TokenKind::Rparen);

        nodes
    }

    /// Depednecy path parsing
    fn dependency_path(&mut self) -> DependencyPath {
        // module name string
        let mut module = EcoString::new();
        // start token, used to create span
        let start = self.peek().clone();

        // first `id`
        module.push_str(&self.consume(TokenKind::Id).value.clone());

        // while path separator exists, parsing new segment
        while self.check(TokenKind::Slash) {
            self.consume(TokenKind::Slash);
            module.push('/');
            module.push_str(&self.consume(TokenKind::Id).value.clone());
        }

        // end token, used to create span
        let end = self.previous().clone();

        DependencyPath {
            address: Address::span(start.address.span.start..end.address.span.end),
            module,
        }
    }

    /// Type annotation parsing
    fn type_annotation(&mut self) -> TypePath {
        // If function type annotation
        if self.check(TokenKind::Fn) {
            // start of span `fn (...): ...`
            let span_start = self.peek().address.clone();
            self.consume(TokenKind::Fn);
            // params
            self.consume(TokenKind::Lparen);
            let mut params = Vec::new();
            params.push(self.type_annotation());
            while self.check(TokenKind::Comma) {
                self.advance();
                params.push(self.type_annotation());
            }
            self.consume(TokenKind::Rparen);
            // : $ret
            self.consume(TokenKind::Colon);
            let ret = Box::new(self.type_annotation());
            // end of span `fn (...): ...`
            let span_end = self.peek().address.clone();
            // function type path
            TypePath::Function {
                location: Address::span(span_start.span.start..span_end.span.end),
                params,
                ret,
            }
        }
        // Else, type name annotation
        else {
            // fisrt id
            let first_id = self.consume(TokenKind::Id).clone();
            // if dot found
            if self.check(TokenKind::Dot) {
                // consuming dot
                self.consume(TokenKind::Dot);
                // module type path
                TypePath::Module {
                    location: first_id.address,
                    module: first_id.value,
                    name: self.consume(TokenKind::Id).value.clone(),
                }
            }
            // else
            else {
                // local type path
                TypePath::Local {
                    location: first_id.address,
                    name: first_id.value,
                }
            }
        }
    }

    /// Single parameter parsing
    fn parameter(&mut self) -> Parameter {
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

    /// Parameters parsing `($name: $type, $name: $type, n )`
    fn parameters(&mut self) -> Vec<Parameter> {
        // result list
        let mut params: Vec<Parameter> = Vec::new();

        // `($name: $type, $name: $type, n )`
        self.consume(TokenKind::Lparen);
        if !self.check(TokenKind::Rparen) {
            params.push(self.parameter());

            while self.check(TokenKind::Comma) {
                self.consume(TokenKind::Comma);
                params.push(self.parameter());
            }
        }
        self.consume(TokenKind::Rparen);

        params
    }

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
        let end_span = self.peek().address.clone();

        // body
        let body = self.block();

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
        let body = self.block();
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
        let body = self.block();
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
        let body = self.block();
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
            body: body,
            else_branches,
        }
    }

    /// Variable parsing
    fn variable(&mut self) -> Expression {
        // start address
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
                let address = Address::span(start_address.span.start..end_address.span.end - 1);
                result = Expression::Call {
                    location: address,
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

    /// Assignment parsing
    fn assignment(&mut self, address: Address, variable: Expression) -> Statement {
        match variable {
            Expression::Call { location, .. } => bail!(ParseError::InvalidAssignmentOperation {
                src: self.named_source.clone(),
                span: location.span.into()
            }),
            _ => {
                let op = self.peek().clone();
                match op.tk_type {
                    TokenKind::Assign => {
                        let start_address = self.advance().address.clone();
                        let expr = self.expr();
                        let end_address = self.peek().address.clone();
                        Statement::VarAssign {
                            location: start_address + end_address,
                            what: variable,
                            value: expr,
                        }
                    }
                    TokenKind::AddAssign => {
                        let op_address = self.advance().address.clone();
                        let expr = Box::new(self.expr());
                        let end_address = self.peek().address.clone();
                        Statement::VarAssign {
                            location: op_address.clone() + end_address.clone(),
                            what: variable.clone(),
                            value: Expression::Bin {
                                location: op_address + end_address,
                                left: Box::new(variable),
                                right: expr,
                                op: BinaryOp::Add,
                            },
                        }
                    }
                    TokenKind::SubAssign => {
                        let op_address = self.advance().address.clone();
                        let expr = Box::new(self.expr());
                        let end_address = self.peek().address.clone();
                        Statement::VarAssign {
                            location: Address::span(address.span.start..end_address.span.end - 1),
                            what: variable.clone(),
                            value: Expression::Bin {
                                location: op_address + end_address,
                                left: Box::new(variable),
                                right: expr,
                                op: BinaryOp::Sub,
                            },
                        }
                    }
                    TokenKind::MulAssign => {
                        let op_address = self.advance().address.clone();
                        let expr = Box::new(self.expr());
                        let end_address = self.peek().address.clone();
                        Statement::VarAssign {
                            location: Address::span(address.span.start..end_address.span.end - 1),
                            what: variable.clone(),
                            value: Expression::Bin {
                                location: op_address + end_address,
                                left: Box::new(variable),
                                right: expr,
                                op: BinaryOp::Mul,
                            },
                        }
                    }
                    TokenKind::DivAssign => {
                        let op_address = self.advance().address.clone();
                        let expr = Box::new(self.expr());
                        let end_address = self.peek().address.clone();
                        Statement::VarAssign {
                            location: Address::span(address.span.start..end_address.span.end - 1),
                            what: variable.clone(),
                            value: Expression::Bin {
                                location: op_address + end_address,
                                left: Box::new(variable),
                                right: expr,
                                op: BinaryOp::Div,
                            },
                        }
                    }
                    _ => bail!(ParseError::InvalidAssignmentOperator {
                        src: self.named_source.clone(),
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

        Statement::VarDef {
            location: name.address,
            name: name.value,
            typ,
            value,
        }
    }

    /// Grouping expr `( expr )`
    fn grouping_expr(&mut self) -> Expression {
        // `($expr)`
        self.consume(TokenKind::Lparen);
        let expr = self.expr();
        self.consume(TokenKind::Rparen);

        expr
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
            TokenKind::Lparen => self.grouping_expr(),
            TokenKind::Fn => self.anonymous_fn_expr(),
            TokenKind::Match => self.pattern_matching(),
            TokenKind::If => self.if_expr(),
            _ => {
                let token = self.peek().clone();
                bail!(ParseError::UnexpectedExpressionToken {
                    src: self.named_source.clone(),
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
            self.current += 1;
            let right = self.unary_expr();
            let end_location = self.peek().address.clone();
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
            self.current += 1;
            let right = self.multiplicative_expr();
            let end_location = self.peek().address.clone();
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
            let end_location = self.peek().address.clone();
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
            let end_location = self.peek().address.clone();
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
            let end_location = self.peek().address.clone();
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

    /// Expr parsing
    fn expr(&mut self) -> Expression {
        self.logical_expr()
    }

    /// Continue statement parsing
    fn continue_stmt(&mut self) -> Statement {
        let location = self.consume(TokenKind::Continue).address.clone();
        Statement::Continue { location }
    }

    /// Break statement parsing
    fn break_stmt(&mut self) -> Statement {
        let location = self.consume(TokenKind::Break).address.clone();
        Statement::Break { location }
    }

    /// While statement parsing
    fn while_stmt(&mut self) -> Statement {
        let start_span = self.consume(TokenKind::While).address.clone();
        let logical = self.expr();
        let body = self.block();
        let end_span = self.previous().address.clone();

        Statement::While {
            location: start_span + end_span,
            logical,
            body,
        }
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
            // If identifier presented
            else {
                // If dot or paren presented -> enum patterns
                if self.check_next(TokenKind::Dot) || self.check_next(TokenKind::Lparen) {
                    // Parsing variant
                    let value = self.variable();
                    // Checking for unwrap of enum
                    if self.check(TokenKind::Lbrace) {
                        // { .., n fields }
                        self.consume(TokenKind::Lbrace);
                        let mut fields = Vec::new();
                        // Checking for close of braces
                        if self.check(TokenKind::Rbrace) {
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
                        self.consume(TokenKind::Rbrace);
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
        let start = self.peek().address.clone();

        // `match value { patterns, ... }`
        self.consume(TokenKind::Match);
        let value = self.expr();

        // Cases
        self.consume(TokenKind::Lbrace);
        let mut cases = Vec::new();
        while !self.check(TokenKind::Rbrace) {
            // If default
            if self.check_ch("_") {
                // Start address of case
                let start_span = self.consume(TokenKind::Id).address.clone();
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
                    pattern: Pattern::Wildcard,
                    body,
                });
            }
            // If pattern
            else {
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
        }
        self.consume(TokenKind::Rbrace);

        // End address
        let end = self.previous().address.clone();

        Expression::Match {
            location: Address::span(start.span.start..end.span.end),
            value: Box::new(value),
            cases,
        }
    }

    /// Fn declaration parsing
    fn fn_declaration(&mut self, publicity: Publicity) -> Declaration {
        // start location
        let start_location = self.peek().address.clone();
        self.consume(TokenKind::Fn);

        // function name
        let name = self.consume(TokenKind::Id).value.clone();

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

        // body
        let body = self.block();

        // end location
        let end_location = self.previous().address.clone();

        Declaration::Function {
            location: start_location + end_location,
            publicity,
            name,
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

        // body
        self.consume(TokenKind::Assign);
        let body = self.consume(TokenKind::Text).value.clone();

        // end location
        let end_location = self.previous().address.clone();

        Declaration::ExternFunction {
            location: start_location + end_location,
            name,
            publicity,
            params,
            typ,
            body,
        }
    }

    /// Type declaration parsing
    fn type_declaration(&mut self, publicity: Publicity) -> Declaration {
        // start location
        let start_location = self.peek().address.clone();

        // variable is used to create type span in bails.
        let type_tk = self.consume(TokenKind::Type).clone();

        // type name
        let name = self.consume(TokenKind::Id).clone();

        // params
        let mut constructor: Vec<Parameter> = Vec::new();
        if self.check(TokenKind::Lparen) {
            constructor = self.parameters();
        }

        // type contents
        let mut declarations = Vec::new();

        // body parsing
        if self.check(TokenKind::Lbrace) {
            self.consume(TokenKind::Lbrace);
            while !self.check(TokenKind::Rbrace) {
                let location = self.peek().clone();
                match self.peek().tk_type {
                    TokenKind::Fn => declarations.push(self.fn_declaration(Publicity::Private)),
                    TokenKind::Let => declarations.push(self.let_declaration(Publicity::Private)),
                    TokenKind::Pub => {
                        // pub
                        self.consume(TokenKind::Pub);
                        // let or fn declaration
                        match self.peek().tk_type {
                            TokenKind::Fn => {
                                declarations.push(self.fn_declaration(Publicity::Public))
                            }
                            TokenKind::Let => {
                                declarations.push(self.let_declaration(Publicity::Public))
                            }
                            _ => {
                                let end = self.peek().clone();
                                bail!(ParseError::UnexpectedNodeInTypeBody {
                                    src: self.named_source.clone(),
                                    type_span: (type_tk.address + name.address).span.into(),
                                    span: (location.address.span.start..(end.address.span.end - 1))
                                        .into(),
                                })
                            }
                        }
                    }
                    _ => {
                        let end = self.peek().clone();
                        bail!(ParseError::UnexpectedNodeInTypeBody {
                            src: self.named_source.clone(),
                            type_span: (type_tk.address.span.start..name.address.span.end).into(),
                            span: (location.address.span.start..(end.address.span.end - 1)).into(),
                        })
                    }
                }
            }
            self.consume(TokenKind::Rbrace);
        }

        // end location
        let end_location = self.previous().address.clone();

        Declaration::TypeDeclaration {
            location: start_location + end_location,
            publicity,
            name: name.value,
            constructor,
            declarations,
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
                    src: self.named_source.clone(),
                    span: token.address.span.into(),
                    unexpected: token.value
                })
            }
        }
    }

    /// Statement parsing
    fn statement(&mut self) -> Statement {
        // Parsing statement
        let stmt = match self.peek().tk_type {
            TokenKind::Continue => self.continue_stmt(),
            TokenKind::Break => self.break_stmt(),
            TokenKind::While => self.while_stmt(),
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
                        Statement::Expr(self.expr())
                    }
                }
            }
            _ => Statement::Expr(self.expr()),
        };
        // If `;` presented
        if self.check(TokenKind::Semicolon) {
            self.advance();
            Statement::Semi(Box::new(stmt))
        } else {
            stmt
        }
    }

    /*
     helper functions
    */

    /// Gets current token, then adds 1 to current.
    fn advance(&mut self) -> &Token {
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
    fn consume(&mut self, tk_type: TokenKind) -> &Token {
        match self.tokens.get(self.current as usize) {
            Some(tk) => {
                self.current += 1;
                if tk.tk_type == tk_type {
                    tk
                } else {
                    bail!(ParseError::UnexpectedToken {
                        src: self.named_source.clone(),
                        span: tk.address.clone().span.into(),
                        unexpected: tk.value.clone(),
                        expected: tk_type
                    })
                }
            }
            None => bail!(ParseError::UnexpectedEof),
        }
    }

    /// Check current token type is equal to tk_type
    fn check(&self, tk_type: TokenKind) -> bool {
        match self.tokens.get(self.current as usize) {
            Some(tk) => tk.tk_type == tk_type,
            None => false,
        }
    }

    /// Check next token type is equal to tk_type
    fn check_next(&self, tk_type: TokenKind) -> bool {
        match self.tokens.get(self.current as usize + 1) {
            Some(tk) => tk.tk_type == tk_type,
            None => false,
        }
    }

    /// Check current token value is equal to tk_value
    fn check_ch(&self, tk_value: &str) -> bool {
        match self.tokens.get(self.current as usize) {
            Some(tk) => tk.value == tk_value,
            None => false,
        }
    }

    /// Peeks current token, if eof raises error
    fn peek(&self) -> &Token {
        match self.tokens.get(self.current as usize) {
            Some(tk) => tk,
            None => bail!(ParseError::UnexpectedEof),
        }
    }

    /// Peeks previous token, if eof raises error
    fn previous(&self) -> &Token {
        match self.tokens.get((self.current - 1) as usize) {
            Some(tk) => tk,
            None => bail!(ParseError::UnexpectedEof),
        }
    }

    /// Check `self.current >= self.tokens.len()`
    fn is_at_end(&self) -> bool {
        self.current as usize >= self.tokens.len()
    }
}
