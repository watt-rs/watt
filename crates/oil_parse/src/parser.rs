/// Imports
use crate::errors::ParseError;
use miette::NamedSource;
use oil_ast::ast::*;
use oil_common::address::Address;
use oil_common::bail;
use oil_lex::tokens::{Token, TokenKind};

/// Parser structure
pub struct Parser<'file_path> {
    tokens: Vec<Token>,
    current: u128,
    named_source: &'file_path NamedSource<String>,
}
/// Parser implementation
#[allow(unused_qualifications)]
impl<'file_path> Parser<'file_path> {
    /// New parser
    pub fn new(tokens: Vec<Token>, named_source: &'file_path NamedSource<String>) -> Self {
        Parser {
            tokens,
            current: 0,
            named_source,
        }
    }

    /// Parsing all declarations
    pub fn parse(&mut self) -> Node {
        // parsing declaration before reaching
        // end of file
        let mut nodes: Vec<Node> = Vec::new();
        while !self.is_at_end() {
            match self.peek().tk_type {
                TokenKind::Pub => {
                    self.consume(TokenKind::Pub);
                    nodes.push(self.declaration(Publicity::Public))
                }
                TokenKind::Use => {
                    nodes.push(self.use_declaration());
                }
                _ => nodes.push(self.declaration(Publicity::Private)),
            }
        }

        Node::Block { body: nodes }
    }

    /// Block statement parsing
    fn block(&mut self) -> Node {
        // parsing statement before reaching
        // end of file, or a `}`
        let mut nodes: Vec<Node> = Vec::new();
        while !self.check(TokenKind::Rbrace) {
            nodes.push(self.statement());
        }

        Node::Block { body: nodes }
    }

    /// Arguments parsing `($expr, $expr, n...)`
    fn args(&mut self) -> Vec<Node> {
        // result list
        let mut nodes: Vec<Node> = Vec::new();

        // `(Node, Node, n...)`
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
        // creating new segments list
        let mut segments_list = Vec::new();

        // start token, used to create span
        let start = self.peek().address.clone();

        // first `id`
        segments_list.push(DependencyPathSegment {
            identifier: self.consume(TokenKind::Id).value.clone(),
        });

        // while path separator exists, parsing new segment
        while self.check(TokenKind::Slash) {
            self.consume(TokenKind::Slash);
            segments_list.push(DependencyPathSegment {
                identifier: self.consume(TokenKind::Id).value.clone(),
            });
        }

        // end token, used to create span
        let end = self.previous().address.clone();

        DependencyPath::new(
            Address::span(start.span.start..end.span.end, start.file.unwrap()),
            segments_list,
        )
    }

    /// Type annotation parsing
    fn type_annotation(&mut self) -> TypePath {
        // fisrt id
        let first_id = self.consume(TokenKind::Id).value.clone();
        // if dot found
        if self.check(TokenKind::Dot) {
            // consuming dot
            self.consume(TokenKind::Dot);
            // module type path
            TypePath::Module {
                module: first_id,
                name: self.consume(TokenKind::Id).value.clone(),
            }
        }
        // else
        else {
            // local type path
            TypePath::Local(first_id)
        }
    }

    /// Single parameter parsing
    fn parameter(&mut self) -> Parameter {
        // `$name: $typ`
        let name = self.consume(TokenKind::Id).clone();
        self.consume(TokenKind::Colon);
        let typ = self.type_annotation();

        Parameter { name, typ }
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

    /// Access part
    fn access_part(&mut self, previous: Option<Box<Node>>) -> Node {
        // if identifier is found
        if self.check(TokenKind::Id) {
            // new indetifier
            let identifier = self.consume(TokenKind::Id).clone();
            // call value with `( args )`
            if self.check(TokenKind::Lparen) {
                let args = self.args();
                Node::Call {
                    previous,
                    name: identifier,
                    args,
                }
            }
            // get variable value
            else {
                return Node::Get {
                    previous,
                    name: identifier,
                };
            }
        }
        // if not, raising an error
        else {
            let tk = self.peek().clone();
            bail!(ParseError::ExpectedIdAsAccess {
                src: self.named_source.clone(),
                span: tk.address.span.into(),
                unexpected: tk.value
            });
        }
    }

    /// Access parsing
    fn access(&mut self, is_expr: bool) -> Node {
        // left
        let start_address = self.peek().address.clone();
        let mut left = self.access_part(Option::None);

        // by dot
        while self.check(TokenKind::Dot) {
            self.consume(TokenKind::Dot);
            left = self.access_part(Option::Some(Box::new(left)));
        }

        // getting end token
        let end_address = self.previous().address.clone();

        // address of full access
        let address = Address::span(
            start_address.span.start..end_address.span.end - 1,
            start_address.file.unwrap(),
        );

        // if is epxression
        if is_expr {
            // checking for wrong assignment
            if self.check(TokenKind::Assign)
                || self.check(TokenKind::AddAssign)
                || self.check(TokenKind::SubAssign)
                || self.check(TokenKind::MulAssign)
                || self.check(TokenKind::DivAssign)
            {
                let tk = self.advance().clone();
                bail!(ParseError::InvalidOperationInExpr {
                    src: self.named_source.clone(),
                    span: tk.address.span.into(),
                    unexpected: tk.value
                })
            }
        }
        // else, if statement
        else {
            // checking for assignment `=`
            if self.check(TokenKind::Assign) {
                self.consume(TokenKind::Assign);
                let value = self.expr();
                match left {
                    Node::Get { previous, name, .. } => {
                        return Node::Assign {
                            previous: previous,
                            name: name,
                            value: Box::new(value),
                        };
                    }
                    _ => {
                        bail!(ParseError::InvalidAssignOperation {
                            src: self.named_source.clone(),
                            span: address.span.into()
                        })
                    }
                }
            }
            // checking for assignments `+=`, `-=`, `/=`, `*=`
            if self.check(TokenKind::AddAssign)
                || self.check(TokenKind::SubAssign)
                || self.check(TokenKind::MulAssign)
                || self.check(TokenKind::DivAssign)
            {
                let address = self.peek().address.clone();

                let (op, op_kind) = match self.advance().tk_type {
                    TokenKind::AddAssign => ("+", TokenKind::Plus),
                    TokenKind::SubAssign => ("-", TokenKind::Minus),
                    TokenKind::MulAssign => ("*", TokenKind::Star),
                    TokenKind::DivAssign => ("/", TokenKind::Slash),
                    kind => bail!(ParseError::UnexpectedAssignmentOperator { unexpected: kind }),
                };

                match left {
                    Node::Get { previous, name, .. } => {
                        let variable = Node::Get {
                            previous: previous.clone(),
                            name: name.clone(),
                        };
                        return Node::Assign {
                            previous,
                            name: name,
                            value: Box::new(Node::Bin {
                                left: Box::new(variable),
                                right: Box::new(self.expr()),
                                op: Token::new(op_kind, op.to_string(), address),
                            }),
                        };
                    }
                    _ => {
                        bail!(ParseError::InvalidCompoundOperation {
                            src: self.named_source.clone(),
                            span: address.span.into(),
                            op
                        })
                    }
                };
            }
        }

        // returning
        left
    }

    /// Let declaration parsing
    fn let_declaration(&mut self, publicity: Publicity) -> Node {
        // `let $id`
        self.consume(TokenKind::Let);
        let name = self.consume(TokenKind::Id).clone();

        // type
        let typ;

        // if specified
        if self.check(TokenKind::Colon) {
            // `: $type`
            self.consume(TokenKind::Colon);
            typ = Option::Some(self.type_annotation());
        }
        // else
        else {
            // setting type to None
            typ = Option::None
        }

        // `= $value`
        self.consume(TokenKind::Assign);
        let value = self.expr();

        return Node::Define {
            publicity,
            name,
            typ,
            value: Box::new(value),
        };
    }

    /// Grouping expr `( expr )`
    fn grouping_expr(&mut self) -> Node {
        // `($expr)`
        self.consume(TokenKind::Lparen);
        let expr = self.expr();
        self.consume(TokenKind::Rparen);

        expr
    }

    /// Primary expr parsing
    fn primary_expr(&mut self) -> Node {
        match self.peek().tk_type {
            TokenKind::Id => self.access(true),
            TokenKind::Number => Node::Number {
                value: self.consume(TokenKind::Number).clone(),
            },
            TokenKind::Text => Node::String {
                value: self.consume(TokenKind::Text).clone(),
            },
            TokenKind::Bool => Node::Bool {
                value: self.consume(TokenKind::Bool).clone(),
            },
            TokenKind::Lparen => self.grouping_expr(),
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
    fn unary_expr(&mut self) -> Node {
        if self.check(TokenKind::Bang) || self.check(TokenKind::Minus) {
            let op = self.advance().clone();

            Node::Unary {
                op,
                value: Box::new(self.primary_expr()),
            }
        } else {
            self.primary_expr()
        }
    }

    /// Binary operations `*`, `/`, `%`, `^`, `&`, `|` parsing
    fn multiplicative_expr(&mut self) -> Node {
        let mut left = self.unary_expr();

        while self.check(TokenKind::Slash)
            || self.check(TokenKind::Star)
            || self.check(TokenKind::BitwiseAnd)
            || self.check(TokenKind::BitwiseOr)
            || self.check(TokenKind::Percent)
            || self.check(TokenKind::Or)
        {
            let op = self.peek().clone();
            self.current += 1;
            let right = self.unary_expr();
            left = Node::Bin {
                left: Box::new(left),
                right: Box::new(right),
                op,
            }
        }

        left
    }

    /// Binary operations `+`, `-` parsing
    fn additive_expr(&mut self) -> Node {
        let mut left = self.multiplicative_expr();

        while self.check(TokenKind::Plus) || self.check(TokenKind::Minus) {
            let op = self.peek().clone();
            self.current += 1;
            let right = self.multiplicative_expr();
            left = Node::Bin {
                left: Box::new(left),
                right: Box::new(right),
                op,
            }
        }

        left
    }

    /// Range expr parsing `n..k`
    fn range_expr(&mut self) -> Node {
        let mut left = self.additive_expr();

        if self.check(TokenKind::Range) {
            let location = self.consume(TokenKind::Range).clone();
            let right = self.additive_expr();
            left = Node::Range {
                location,
                from: Box::new(left),
                to: Box::new(right),
            }
        }

        left
    }

    /// Compare operations `<`, `>`, `<=`, `>=` parsing
    fn compare_expr(&mut self) -> Node {
        let mut left = self.range_expr();

        if self.check(TokenKind::Greater)
            || self.check(TokenKind::Less)
            || self.check(TokenKind::LessEq)
            || self.check(TokenKind::GreaterEq)
        {
            let op = self.advance().clone();
            let right = self.range_expr();
            left = Node::Cond {
                left: Box::new(left),
                right: Box::new(right),
                op,
            };
        }

        left
    }

    /// Equality operations `==`, `!=` parsing
    fn equality_expr(&mut self) -> Node {
        let mut left = self.compare_expr();

        if self.check(TokenKind::Eq) || self.check(TokenKind::NotEq) {
            let op = self.advance().clone();
            let right = self.compare_expr();
            left = Node::Cond {
                left: Box::new(left),
                right: Box::new(right),
                op,
            };
        }

        left
    }

    /// Logical operations `and`, `or` parsing
    fn logical_expr(&mut self) -> Node {
        let mut left = self.equality_expr();

        while self.check(TokenKind::And) || self.check(TokenKind::Or) {
            let op = self.advance().clone();
            let right = self.equality_expr();
            left = Node::Logical {
                left: Box::new(left),
                right: Box::new(right),
                op,
            };
        }

        left
    }

    /// Expr parsing
    fn expr(&mut self) -> Node {
        self.logical_expr()
    }

    /// Continue statement parsing
    fn continue_stmt(&mut self) -> Node {
        let location = self.consume(TokenKind::Continue).clone();
        Node::Continue { location }
    }

    /// Break statement parsing
    fn break_stmt(&mut self) -> Node {
        let location = self.consume(TokenKind::Break).clone();
        Node::Break { location }
    }

    /// Return statement parsing
    fn return_stmt(&mut self) -> Node {
        let location = self.consume(TokenKind::Ret).clone();
        let value = Box::new(self.expr());
        Node::Return { location, value }
    }

    /// Use declaration `use ...` | `use (..., ..., n)` parsing
    fn use_declaration(&mut self) -> Node {
        self.consume(TokenKind::Use);

        // `path/to/module`
        let path = self.dependency_path();
        let name;
        // `as $id`
        if self.check(TokenKind::As) {
            self.consume(TokenKind::As);
            name = Option::Some(self.consume(TokenKind::Id).clone());
        } else {
            name = Option::None;
        }

        Node::Use { path, name }
    }

    /// While statement parsing
    fn while_stmt(&mut self) -> Node {
        let location = self.consume(TokenKind::While).clone();
        let logical = self.expr();

        self.consume(TokenKind::Lbrace);
        let body = self.block();
        self.consume(TokenKind::Rbrace);

        Node::While {
            location,
            logical: Box::new(logical),
            body: Box::new(body),
        }
    }

    /// Else parsing
    fn else_stmt(&mut self) -> Node {
        let location = self.consume(TokenKind::Else).clone();

        self.consume(TokenKind::Lbrace);
        let body = self.block();
        self.consume(TokenKind::Rbrace);

        Node::If {
            location: location.clone(),
            logical: Box::new(Node::Bool {
                value: Token::new(TokenKind::Bool, "true".to_string(), location.address),
            }),
            body: Box::new(body),
            elseif: None,
        }
    }

    /// Elif parsing
    fn elif_stmt(&mut self) -> Node {
        let location = self.consume(TokenKind::Elif).clone();
        let logical = self.expr();

        self.consume(TokenKind::Lbrace);
        let body = self.block();
        self.consume(TokenKind::Rbrace);

        // if elif / else is passed
        if self.check(TokenKind::Elif) {
            Node::If {
                location,
                logical: Box::new(logical),
                body: Box::new(body),
                elseif: Some(Box::new(self.elif_stmt())),
            }
        } else if self.check(TokenKind::Else) {
            Node::If {
                location,
                logical: Box::new(logical),
                body: Box::new(body),
                elseif: Some(Box::new(self.else_stmt())),
            }
        } else {
            Node::If {
                location,
                logical: Box::new(logical),
                body: Box::new(body),
                elseif: None,
            }
        }
    }

    /// If statement parsing
    fn if_stmt(&mut self) -> Node {
        let location = self.consume(TokenKind::If).clone();
        let logical = self.expr();

        self.consume(TokenKind::Lbrace);
        let body = self.block();
        self.consume(TokenKind::Rbrace);

        // if elif / else is passed
        if self.check(TokenKind::Elif) {
            Node::If {
                location,
                logical: Box::new(logical),
                body: Box::new(body),
                elseif: Some(Box::new(self.elif_stmt())),
            }
        } else if self.check(TokenKind::Else) {
            Node::If {
                location,
                logical: Box::new(logical),
                body: Box::new(body),
                elseif: Some(Box::new(self.else_stmt())),
            }
        } else {
            Node::If {
                location,
                logical: Box::new(logical),
                body: Box::new(body),
                elseif: None,
            }
        }
    }

    /// For statement parsing
    fn for_stmt(&mut self) -> Node {
        // `for i in $expr`
        self.consume(TokenKind::For);
        let name = self.consume(TokenKind::Id).clone();
        self.consume(TokenKind::In);
        let value = self.expr();

        // body
        self.consume(TokenKind::Lbrace);
        let body = self.block();
        self.consume(TokenKind::Rbrace);

        Node::For {
            variable: name,
            iterable: Box::new(value),
            body: Box::new(body),
        }
    }

    /// Fn declaration parsing
    fn fn_declaration(&mut self, publicity: Publicity) -> Node {
        self.consume(TokenKind::Fn);

        // function name
        let name = self.consume(TokenKind::Id).clone();

        // params
        let mut params: Vec<Parameter> = Vec::new();
        if self.check(TokenKind::Lparen) {
            params = self.parameters();
        }

        // return type
        let typ;
        // if type specified
        if self.check(TokenKind::Colon) {
            // `: $type`
            self.consume(TokenKind::Colon);
            typ = Some(self.type_annotation());
        }
        // else
        else {
            typ = None
        }

        // body
        self.consume(TokenKind::Lbrace);
        let body = self.block();
        self.consume(TokenKind::Rbrace);

        Node::FnDeclaration {
            publicity,
            name,
            params,
            body: Box::new(body),
            typ,
        }
    }

    /// Type declaration parsing
    fn type_declaration(&mut self, publicity: Publicity) -> Node {
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
        let mut functions = Vec::new();
        let mut fields = Vec::new();

        // body parsing
        if self.check(TokenKind::Lbrace) {
            self.consume(TokenKind::Lbrace);
            while !self.check(TokenKind::Rbrace) {
                let location = self.peek().clone();
                match self.peek().tk_type {
                    TokenKind::Fn => functions.push(self.fn_declaration(Publicity::Private)),
                    TokenKind::Let => fields.push(self.let_declaration(Publicity::Private)),
                    TokenKind::Pub => {
                        // pub
                        self.consume(TokenKind::Pub);
                        // let or fn declaration
                        match self.peek().tk_type {
                            TokenKind::Fn => functions.push(self.fn_declaration(Publicity::Public)),
                            TokenKind::Let => fields.push(self.let_declaration(Publicity::Public)),
                            _ => {
                                let end = self.peek().clone();
                                bail!(ParseError::UnexpectedNodeInTypeBody {
                                    src: self.named_source.clone(),
                                    type_span: (type_tk.address.span.start..name.address.span.end)
                                        .into(),
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

        Node::TypeDeclaration {
            publicity,
            name,
            constructor,
            functions,
            fields,
        }
    }

    /// Declaration parsing
    fn declaration(&mut self, publicity: Publicity) -> Node {
        match self.peek().tk_type {
            TokenKind::Type => self.type_declaration(publicity),
            TokenKind::Fn => self.fn_declaration(publicity),
            TokenKind::Let => self.let_declaration(publicity),
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
    fn statement(&mut self) -> Node {
        match self.peek().tk_type {
            TokenKind::If => self.if_stmt(),
            TokenKind::Id => self.access(false),
            TokenKind::Continue => self.continue_stmt(),
            TokenKind::Break => self.break_stmt(),
            TokenKind::Ret => self.return_stmt(),
            TokenKind::For => self.for_stmt(),
            TokenKind::While => self.while_stmt(),
            TokenKind::Let => self.let_declaration(Publicity::None),
            TokenKind::Fn => self.fn_declaration(Publicity::None),
            _ => {
                let token = self.peek().clone();
                bail!(ParseError::UnexpectedStatementToken {
                    src: self.named_source.clone(),
                    span: token.address.span.into(),
                    unexpected: token.value,
                });
            }
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
