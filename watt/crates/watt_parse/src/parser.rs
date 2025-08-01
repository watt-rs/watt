// imports
use std::path::PathBuf;
use watt_ast::ast::*;
use watt_ast::import::Import;
use watt_common::address::Address;
use watt_common::{error, errors::Error};
use watt_lex::tokens::{Token, TokenKind};

/// Parser structure
pub struct Parser<'file_path, 'prefix> {
    tokens: Vec<Token>,
    current: u128,
    file_path: &'file_path PathBuf,
    full_name_prefix: &'prefix str,
}
/// Parser implementation
#[allow(unused_qualifications)]
impl<'file_path, 'prefix> Parser<'file_path, 'prefix> {
    /// New parser
    pub fn new(
        tokens: Vec<Token>,
        file_path: &'file_path PathBuf,
        full_name_prefix: &'prefix str,
    ) -> Self {
        Parser {
            tokens,
            current: 0,
            file_path,
            full_name_prefix,
        }
    }

    /// Block statement parsing
    fn block(&mut self) -> Node {
        let mut nodes: Vec<Node> = Vec::new();
        while !self.is_at_end() && !self.check(TokenKind::Rbrace) {
            nodes.push(self.statement());
        }
        Node::Block { body: nodes }
    }

    /// Arguments parsing `( Node, Node, n )`
    fn args(&mut self) -> Vec<Node> {
        let mut nodes: Vec<Node> = Vec::new();
        self.consume(TokenKind::Lparen);

        if !self.check(TokenKind::Rparen) {
            nodes.push(self.expr());
            while !self.is_at_end() && self.check(TokenKind::Comma) {
                self.consume(TokenKind::Comma);
                nodes.push(self.expr());
            }
        }
        self.consume(TokenKind::Rparen);

        nodes
    }

    /// Parameters parsing `( Token, Token, n )`
    fn params(&mut self) -> Vec<Token> {
        let mut nodes: Vec<Token> = Vec::new();
        self.consume(TokenKind::Lparen);

        if !self.check(TokenKind::Rparen) {
            nodes.push(self.consume(TokenKind::Id).clone());
            while !self.is_at_end() && self.check(TokenKind::Comma) {
                self.consume(TokenKind::Comma);
                nodes.push(self.consume(TokenKind::Id).clone());
            }
        }
        self.consume(TokenKind::Rparen);

        nodes
    }

    /// Converts name to full name, using pattern:
    /// `test_fn` from file test.wt is converted to `test:test_fn`
    fn to_full_name(&self, tk: Token) -> Token {
        Token::new(
            TokenKind::Text,
            format!("{}:{}", self.full_name_prefix, tk.value),
            tk.address,
        )
    }

    /// Object creation expr
    fn object_creation_expr(&mut self) -> Node {
        self.consume(TokenKind::New);

        let name = self.consume(TokenKind::Id).clone();
        let args = self.args();

        Node::Instance {
            name,
            constructor: args,
            should_push: true,
        }
    }

    /// Access expr part
    fn access_part(&mut self, previous: Option<Box<Node>>) -> Node {
        if self.check(TokenKind::Id) {
            let identifier = self.consume(TokenKind::Id).clone();
            // :=
            if self.check(TokenKind::Walrus) {
                self.consume(TokenKind::Walrus);
                Node::Define {
                    previous,
                    name: identifier,
                    value: Box::new(self.expr()),
                }
            }
            // =
            else if self.check(TokenKind::Assign) {
                self.consume(TokenKind::Assign);
                Node::Assign {
                    previous,
                    name: identifier,
                    value: Box::new(self.expr()),
                }
            }
            // +=, -=, *=, /=
            else if self.check(TokenKind::AssignAdd)
                || self.check(TokenKind::AssignSub)
                || self.check(TokenKind::AssignMul)
                || self.check(TokenKind::AssignDiv)
            {
                let op;
                let location;
                match self.peek().tk_type {
                    TokenKind::AssignSub => {
                        location = self.consume(TokenKind::AssignSub).clone();
                        op = "-";
                    }
                    TokenKind::AssignMul => {
                        location = self.consume(TokenKind::AssignMul).clone();
                        op = "*";
                    }
                    TokenKind::AssignDiv => {
                        location = self.consume(TokenKind::AssignDiv).clone();
                        op = "/";
                    }
                    TokenKind::AssignAdd => {
                        location = self.consume(TokenKind::AssignAdd).clone();
                        op = "+";
                    }
                    _ => {
                        panic!("invalid AssignOp tk_type. report to developer.");
                    }
                }
                let var = Node::Get {
                    previous: previous.clone(),
                    name: identifier.clone(),
                    should_push: true,
                };
                return Node::Assign {
                    previous,
                    name: identifier,
                    value: Box::new(Node::Bin {
                        left: Box::new(var),
                        right: Box::new(self.expr()),
                        op: Token::new(TokenKind::Op, op.to_string(), location.address),
                    }),
                };
            }
            // ( args )
            else if self.check(TokenKind::Lparen) {
                let args = self.args();
                return if self.check(TokenKind::Question) {
                    self.consume(TokenKind::Question);
                    Node::ErrorPropagation {
                        location: identifier.clone(),
                        value: Box::new(Node::Call {
                            previous,
                            name: identifier,
                            args,
                            should_push: true,
                        }),
                        should_push: true,
                    }
                } else {
                    Node::Call {
                        previous,
                        name: identifier,
                        args,
                        should_push: true,
                    }
                };
            }
            // get
            else {
                return Node::Get {
                    previous,
                    name: identifier,
                    should_push: true,
                };
            }
        }
        // object creation
        else {
            self.object_creation_expr()
        }
    }

    /// Access parsing
    /// if is_expr should_push will be true
    /// else should_push will be false
    ///
    fn access(&mut self, is_expr: bool) -> Node {
        // left
        let mut left = self.access_part(Option::None);

        // by dot
        while self.check(TokenKind::Dot) {
            self.consume(TokenKind::Dot);
            let location = self.peek().address.clone();
            left = self.access_part(Option::Some(Box::new(left)));
            if !is_expr {
                continue;
            }
            match left {
                Node::Define { .. } => {
                    error!(Error::new(
                        location,
                        "couldn't use define in expr.",
                        "check your code.",
                    ));
                }
                Node::Assign { .. } => {
                    error!(Error::new(
                        location,
                        "couldn't use assign in expr.",
                        "check your code.",
                    ));
                }
                _ => {}
            }
        }
        left = set_should_push(left, is_expr);

        left
    }

    /// Access expr parsing
    fn access_expr(&mut self) -> Node {
        self.access(true)
    }

    /// Access statement parsing
    fn access_stmt(&mut self) -> Node {
        self.access(false)
    }

    /// Grouping expr `( expr )`
    fn grouping_expr(&mut self) -> Node {
        self.consume(TokenKind::Lparen);
        let expr = self.expr();
        self.consume(TokenKind::Rparen);
        expr
    }

    /// Anonymous fn parsing
    fn anonymous_fn_expr(&mut self) -> Node {
        let location = self.consume(TokenKind::Fn).clone();

        // params
        let mut params: Vec<Token> = Vec::new();
        if self.check(TokenKind::Lparen) {
            params = self.params();
        }

        // body
        self.consume(TokenKind::Lbrace);
        let body = self.block();
        self.consume(TokenKind::Rbrace);

        Node::AnFnDeclaration {
            location,
            params,
            body: Box::new(body),
            make_closure: false,
        }
    }

    /// Lambda expr parsing
    fn lambda_fn_expr(&mut self) -> Node {
        let location = self.consume(TokenKind::Lambda).clone();

        // params
        let mut params: Vec<Token> = Vec::new();
        if self.check(TokenKind::Lparen) {
            params = self.params();
        }

        // ->
        self.consume(TokenKind::Arrow);

        // body
        let body = self.expr();

        Node::AnFnDeclaration {
            location: location.clone(),
            params,
            body: Box::new(Node::Ret {
                location,
                value: Box::new(body),
            }),
            make_closure: false,
        }
    }

    /// Primary expr parsing
    fn primary_expr(&mut self) -> Node {
        match self.peek().tk_type {
            TokenKind::Id | TokenKind::New => self.access_expr(),
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
            TokenKind::Lbrace => self.map_expr(),
            TokenKind::Lbracket => self.list_expr(),
            TokenKind::Null => Node::Null {
                location: self.consume(TokenKind::Null).clone(),
            },
            TokenKind::Fn => self.anonymous_fn_expr(),
            TokenKind::Lambda => self.lambda_fn_expr(),
            TokenKind::Match => self.match_expr(),
            _ => error!(Error::own_text(
                self.peek().address.clone(),
                format!(
                    "invalid token. {:?}:{:?}",
                    self.peek().tk_type,
                    self.peek().value
                ),
                "check your code.",
            )),
        }
    }

    /// Match expr parsing
    fn match_expr(&mut self) -> Node {
        let location = self.consume(TokenKind::Match).clone();
        // matchable
        let matchable = self.expr();

        // cases
        let mut cases = vec![];
        let default;

        /// Makes lambda from body
        fn make_lambda(location: Token, body: Node) -> Node {
            Node::Block {
                body: vec![
                    Node::Define {
                        previous: None,
                        name: Token::new(
                            TokenKind::Id,
                            "@match_lambda".to_string(),
                            location.address.clone(),
                        ),
                        value: Box::new(Node::AnFnDeclaration {
                            location: location.clone(),
                            params: vec![],
                            body: Box::new(body),
                            make_closure: false,
                        }),
                    },
                    Node::Call {
                        previous: None,
                        name: Token::new(
                            TokenKind::Id,
                            "@match_lambda".to_string(),
                            location.address.clone(),
                        ),
                        args: vec![],
                        should_push: true,
                    },
                ],
            }
        }

        // cases body
        self.consume(TokenKind::Lbrace);
        while self.check(TokenKind::Case) {
            self.consume(TokenKind::Case);
            let value = self.expr();

            // one line
            if self.check(TokenKind::Arrow) {
                self.consume(TokenKind::Arrow);
                cases.push(MatchCase::new(Box::new(value), Box::new(self.expr())));
            }
            // multi line
            else if self.check(TokenKind::Lbrace) {
                self.consume(TokenKind::Lbrace);
                let body = self.block();
                self.consume(TokenKind::Rbrace);
                cases.push(MatchCase::new(
                    Box::new(value),
                    Box::new(make_lambda(location.clone(), body)),
                ));
            } else {
                error!(Error::new(
                    location.address.clone(),
                    "expected arrow or brace after case value",
                    "check your code",
                ));
            }
        }
        // default case
        self.consume(TokenKind::Default);

        // one line
        if self.check(TokenKind::Arrow) {
            self.consume(TokenKind::Arrow);
            default = Box::new(self.expr());
        }
        // multi line
        else if self.check(TokenKind::Lbrace) {
            self.consume(TokenKind::Lbrace);
            let body = self.block();
            self.consume(TokenKind::Rbrace);
            default = Box::new(make_lambda(location.clone(), body))
        } else {
            error!(Error::new(
                location.address.clone(),
                "expected arrow or brace after case value",
                "check your code",
            ));
        }
        self.consume(TokenKind::Rbrace);

        Node::Match {
            location,
            matchable: Box::new(matchable),
            cases,
            default,
        }
    }

    /// List expr `[]` parsing
    fn list_expr(&mut self) -> Node {
        let location = self.consume(TokenKind::Lbracket).clone();

        if self.check(TokenKind::Rbracket) {
            self.consume(TokenKind::Rbracket);
            Node::List {
                location,
                values: Vec::new(),
            }
        } else {
            let mut nodes: Vec<Node> = vec![self.expr()];

            while self.check(TokenKind::Comma) {
                self.consume(TokenKind::Comma);
                nodes.push(self.expr());
            }

            self.consume(TokenKind::Rbracket);

            Node::List {
                location,
                values: nodes,
            }
        }
    }

    /// Key value pair `{key: value}` parsing
    fn key_value_expr(&mut self) -> (Node, Node) {
        // key
        let l = self.expr();
        // :
        self.consume(TokenKind::Colon);
        // value
        let r = self.expr();

        (l, r)
    }

    /// Map expr `{pair, n}` parsing
    fn map_expr(&mut self) -> Node {
        let location = self.consume(TokenKind::Lbrace).clone();

        if self.check(TokenKind::Rbrace) {
            self.consume(TokenKind::Rbrace);
            Node::Map {
                location,
                values: Vec::new(),
            }
        } else {
            let mut nodes: Vec<(Node, Node)> = Vec::new();
            let key = self.key_value_expr();

            nodes.push((key.0, key.1));

            while self.check(TokenKind::Comma) {
                self.consume(TokenKind::Comma);
                let key = self.key_value_expr();
                nodes.push((key.0, key.1));
            }

            self.consume(TokenKind::Rbrace);

            Node::Map {
                location,
                values: nodes,
            }
        }
    }

    /// Unary expr `!` and `-` parsing
    fn unary_expr(&mut self) -> Node {
        let tk = self.peek();

        match tk {
            Token { tk_type, value, .. }
                if (tk_type == &TokenKind::Op && value == "-") || (tk_type == &TokenKind::Bang) =>
            {
                let op = self.consume(*tk_type).clone();

                Node::Unary {
                    op,
                    value: Box::new(self.primary_expr()),
                }
            }
            _ => self.primary_expr(),
        }
    }

    /// Binary operations `*`, `/`, `%`, `^`, `&`, `|` parsing
    fn multiplicative_expr(&mut self) -> Node {
        let mut left = self.unary_expr();

        while self.check(TokenKind::Op)
            && (self.peek().value == "*"
                || self.peek().value == "&"
                || self.peek().value == "|"
                || self.peek().value == "^"
                || self.peek().value == "/"
                || self.peek().value == "%")
        {
            let op = self.consume(TokenKind::Op).clone();
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

        while self.check(TokenKind::Op) && (self.peek().value == "+" || self.peek().value == "-") {
            let op = self.consume(TokenKind::Op).clone();
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

    /// Impls expr `a impls b` parsing
    fn impls_expr(&mut self) -> Node {
        let mut left = self.range_expr();

        if self.check(TokenKind::Impls) {
            self.consume(TokenKind::Impls);
            let trait_name = self.consume(TokenKind::Id).clone();
            left = Node::Impls {
                value: Box::new(left),
                trait_name,
            }
        }

        left
    }

    /// Compare operations `<`, `>`, `<=`, `>=` parsing
    fn compare_expr(&mut self) -> Node {
        let mut left = self.impls_expr();

        if self.check(TokenKind::Greater)
            || self.check(TokenKind::Less)
            || self.check(TokenKind::LessEq)
            || self.check(TokenKind::GreaterEq)
        {
            let op = self.peek().clone();
            self.current += 1;
            let right = self.impls_expr();
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
            let op = self.peek().clone();
            self.current += 1;
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
            let op = self.peek().clone();
            self.current += 1;
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
        Node::Ret { location, value }
    }

    /// Single import parsing
    ///
    /// ✔️ With: creates full_name_prefix override
    /// ❌ with: uses default full_name_prefix
    fn single_import(&mut self) -> Import {
        let name = self.consume(TokenKind::Text).clone();
        if self.check(TokenKind::With) {
            self.consume(TokenKind::With);
            Import::new(
                Option::Some(name.address),
                name.value,
                Option::Some(self.consume(TokenKind::Text).value.clone()),
            )
        } else {
            Import::new(Option::Some(name.address), name.value, Option::None)
        }
    }

    /// Import statement `import ...` | `import (..., ..., n)` parsing
    fn import_stmt(&mut self) -> Node {
        let location = self.consume(TokenKind::Import).clone();
        let mut imports = Vec::new();

        // ( import, import, n )
        if self.check(TokenKind::Lparen) {
            self.consume(TokenKind::Lparen);
            imports.push(self.single_import());
            while self.check(TokenKind::Comma) {
                self.consume(TokenKind::Comma);
                imports.push(self.single_import());
            }
        }
        // single import
        else {
            imports.push(self.single_import());
        }

        Node::Import { location, imports }
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

    /// Match statement parsing
    fn match_stmt(&mut self) -> Node {
        let location = self.consume(TokenKind::Match).clone();
        // matchable
        let matchable = self.expr();
        // cases
        let mut cases = vec![];
        let default;
        // body
        self.consume(TokenKind::Lbrace);
        while self.check(TokenKind::Case) {
            self.consume(TokenKind::Case);
            let value = self.expr();
            // one line
            if self.check(TokenKind::Arrow) {
                self.consume(TokenKind::Arrow);
                cases.push(MatchCase::new(Box::new(value), Box::new(self.statement())))
            }
            // multi line
            else if self.check(TokenKind::Lbrace) {
                self.consume(TokenKind::Lbrace);
                let body = self.block();
                self.consume(TokenKind::Rbrace);
                cases.push(MatchCase::new(Box::new(value), Box::new(body)))
            } else {
                error!(Error::new(
                    location.address.clone(),
                    "expected arrow or brace after case value",
                    "check your code",
                ));
            }
        }
        // default
        self.consume(TokenKind::Default);
        // one line
        if self.check(TokenKind::Arrow) {
            self.consume(TokenKind::Arrow);
            default = Box::new(self.statement());
        }
        // multi line
        else if self.check(TokenKind::Lbrace) {
            self.consume(TokenKind::Lbrace);
            let body = self.block();
            self.consume(TokenKind::Rbrace);
            default = Box::new(body);
        } else {
            error!(Error::new(
                location.address.clone(),
                "expected arrow or brace after case value",
                "check your code",
            ));
        }
        self.consume(TokenKind::Rbrace);

        Node::Match {
            location,
            matchable: Box::new(matchable),
            cases,
            default,
        }
    }

    /// For statement parsing
    fn for_stmt(&mut self) -> Node {
        self.consume(TokenKind::For);
        let name = self.consume(TokenKind::Id).clone();
        self.consume(TokenKind::In);
        let value = self.expr();
        self.consume(TokenKind::Lbrace);
        let body = self.block();
        self.consume(TokenKind::Rbrace);
        Node::For {
            variable_name: name,
            iterable: Box::new(value),
            body: Box::new(body),
        }
    }

    /// Fn declaration parsing
    fn function_stmt(&mut self) -> Node {
        self.consume(TokenKind::Fn);

        // fn name
        let name = self.consume(TokenKind::Id).clone();

        // params
        let mut params: Vec<Token> = Vec::new();
        if self.check(TokenKind::Lparen) {
            params = self.params();
        }

        // body
        self.consume(TokenKind::Lbrace);
        let body = self.block();
        self.consume(TokenKind::Rbrace);

        Node::FnDeclaration {
            name: name.clone(),
            full_name: Option::Some(self.to_full_name(name)),
            params,
            body: Box::new(body),
            make_closure: true,
        }
    }

    /// Type declaration parsing
    fn type_stmt(&mut self) -> Node {
        self.consume(TokenKind::Type);

        // type name
        let name = self.consume(TokenKind::Id).clone();

        // params
        let mut constructor: Vec<Token> = Vec::new();
        if self.check(TokenKind::Lparen) {
            constructor = self.params();
        }

        // traits
        let mut impls: Vec<Token> = Vec::new();
        if self.check(TokenKind::Impl) {
            // impls by comma
            self.consume(TokenKind::Impl);
            impls.push(self.consume(TokenKind::Id).clone());
            while !self.is_at_end() && self.check(TokenKind::Comma) {
                self.consume(TokenKind::Comma);
                impls.push(self.consume(TokenKind::Id).clone());
            }
        }
        // body
        let mut body = Vec::new();
        if self.check(TokenKind::Lbrace) {
            self.consume(TokenKind::Lbrace);
            while !self.is_at_end() && !self.check(TokenKind::Rbrace) {
                let location = self.peek().clone();
                let mut node = self.statement();
                match node {
                    Node::FnDeclaration {
                        name, params, body, ..
                    } => {
                        node = Node::FnDeclaration {
                            name,
                            full_name: None,
                            params,
                            body,
                            make_closure: false,
                        }
                    }
                    Node::Native { .. }
                    | Node::Get { .. }
                    | Node::Define { .. }
                    | Node::Assign { .. } => {}
                    _ => {
                        error!(Error::own_text(
                            location.address,
                            format!(
                                "invalid node for type: {:?}:{:?}",
                                location.tk_type, location.value
                            ),
                            "check your code.",
                        ));
                    }
                }
                body.push(node);
            }
            self.consume(TokenKind::Rbrace);
        }

        Node::Type {
            name: name.clone(),
            full_name: Some(self.to_full_name(name)),
            constructor,
            body: Box::new(Node::Block { body }),
            impls,
        }
    }

    /// Trait declaration parsing
    fn trait_stmt(&mut self) -> Node {
        self.consume(TokenKind::Trait);

        // trait name
        let name = self.consume(TokenKind::Id).clone();
        // functions
        let mut functions: Vec<TraitNodeFn> = Vec::new();
        self.consume(TokenKind::Lbrace);
        while !self.is_at_end() && !self.check(TokenKind::Rbrace) {
            let location = self.peek().address.clone();

            if self.check(TokenKind::Fn) {
                self.consume(TokenKind::Fn);

                // function name
                let name = self.consume(TokenKind::Id).clone();

                // params
                let mut params: Vec<Token> = Vec::new();
                if self.check(TokenKind::Lparen) {
                    params = self.params();
                }

                // optional body
                if self.check(TokenKind::Lbrace) {
                    self.consume(TokenKind::Lbrace);
                    let body = self.block();
                    self.consume(TokenKind::Rbrace);

                    functions.push(TraitNodeFn::new(name, params, Option::Some(Box::new(body))))
                } else {
                    functions.push(TraitNodeFn::new(name, params, Option::None))
                }
            } else {
                error!(Error::new(
                    location,
                    "only fn-s can be declared in trait.",
                    "you can create this declaration: 'fn meow(cat)'",
                ))
            }
        }

        self.consume(TokenKind::Rbrace);

        Node::Trait {
            name: name.clone(),
            full_name: Some(self.to_full_name(name)),
            functions,
        }
    }

    /// Unit declaration parsing
    fn unit_stmt(&mut self) -> Node {
        self.consume(TokenKind::Unit);

        // unit name
        let name = self.consume(TokenKind::Id).clone();

        // unit body
        let mut body = Vec::new();
        if self.check(TokenKind::Lbrace) {
            self.consume(TokenKind::Lbrace);
            while !self.is_at_end() && !self.check(TokenKind::Rbrace) {
                let location = self.peek().clone();
                let mut node = self.statement();
                match node {
                    Node::FnDeclaration {
                        name, params, body, ..
                    } => {
                        node = Node::FnDeclaration {
                            name,
                            full_name: None,
                            params,
                            body,
                            make_closure: false,
                        }
                    }
                    Node::Native { .. }
                    | Node::Get { .. }
                    | Node::Define { .. }
                    | Node::Assign { .. } => {}
                    _ => {
                        error!(Error::own_text(
                            location.address,
                            format!(
                                "invalid node for unit: {:?}:{:?}",
                                location.tk_type, location.value
                            ),
                            "check your code.",
                        ));
                    }
                }
                body.push(node);
            }
            self.consume(TokenKind::Rbrace);
        }

        Node::Unit {
            name: name.clone(),
            full_name: Some(self.to_full_name(name)),
            body: Box::new(Node::Block { body }),
        }
    }

    /// Native fn declaration parsing
    fn native_stmt(&mut self) -> Node {
        self.consume(TokenKind::Native);

        // definition name
        let name = self.consume(TokenKind::Id).clone();
        // ->
        self.consume(TokenKind::Arrow);
        // native fn internal name
        let fn_name = self.consume(TokenKind::Text).clone();

        Node::Native { name, fn_name }
    }

    /// Statement parsing
    fn statement(&mut self) -> Node {
        let tk = self.peek();
        match tk.tk_type {
            TokenKind::Type => self.type_stmt(),
            TokenKind::Unit => self.unit_stmt(),
            TokenKind::If => self.if_stmt(),
            TokenKind::New | TokenKind::Id => self.access_stmt(),
            TokenKind::Match => self.match_stmt(),
            TokenKind::Continue => self.continue_stmt(),
            TokenKind::Break => self.break_stmt(),
            TokenKind::Ret => self.return_stmt(),
            TokenKind::Fn => self.function_stmt(),
            TokenKind::Native => self.native_stmt(),
            TokenKind::Import => self.import_stmt(),
            TokenKind::For => self.for_stmt(),
            TokenKind::While => self.while_stmt(),
            TokenKind::Trait => self.trait_stmt(),
            _ => error!(Error::own_text(
                tk.address.clone(),
                format!("unexpected stmt token: {:?}:{}", tk.tk_type, tk.value),
                "check your code.",
            )),
        }
    }

    /// Parsing block
    pub fn parse(&mut self) -> Node {
        self.block()
    }

    /*
     helper functions
    */

    /// Consumes token by kind, if expected kind doesn't equal
    /// current token kind - raises error.
    fn consume(&mut self, tk_type: TokenKind) -> &Token {
        match self.tokens.get(self.current as usize) {
            Some(tk) => {
                self.current += 1;
                if tk.tk_type == tk_type {
                    tk
                } else {
                    error!(Error::own_text(
                        tk.address.clone(),
                        format!(
                            "unexpected token: '{:?}:{}', expected: '{tk_type:?}'",
                            tk.tk_type, tk.value
                        ),
                        "check your code.",
                    ))
                }
            }
            None => error!(Error::new(
                Address::new(0, 0, self.file_path.clone()),
                "unexpected eof",
                "check your code.",
            )),
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
            None => error!(Error::new(
                Address::new(0, 0, self.file_path.clone()),
                "unexpected eof",
                "check your code.",
            )),
        }
    }

    /// Check `self.current >= self.tokens.len()`
    fn is_at_end(&self) -> bool {
        self.current as usize >= self.tokens.len()
    }
}
