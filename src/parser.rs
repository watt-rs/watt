use crate::address::Address;
use crate::ast::{set_should_push, Node};
use crate::errors::{Error, ErrorType};
use crate::lexer::{Token, TokenType};

struct Parser {
    tokens: Vec<Token>,
    current: u128,
    filename: String,
    full_name_prefix: String,
}

impl Parser {
    fn new(filename: String, full_name_prefix: String) -> Parser {
        Parser { tokens: vec![], current: 0, filename, full_name_prefix }
    }

    fn block(&mut self) -> Result<Node, Error> {
        let mut nodes: Vec<Box<Node>> = Vec::new();

        while !self.is_at_end() && !self.check(TokenType::Lbrace) {
            nodes.push(Box::new(self.statement()?));
        }

        Ok(Node::Block {
            body: nodes
        })
    }

    fn args(&mut self) -> Result<Vec<Box<Node>>, Error> {
        let mut nodes: Vec<Box<Node>> = Vec::new();
        self.consume(TokenType::Lparen)?;

        if !self.check(TokenType::Rparen) {
            nodes.push(Box::new(self.expr()?));
            while !self.is_at_end() && self.check(TokenType::Comma) {
                self.consume(TokenType::Comma)?;
                nodes.push(Box::new(self.expr()?));
            }
        }

        self.consume(TokenType::Rparen)?;
        Ok(nodes)
    }

    fn params(&mut self) -> Result<Vec<Token>, Error> {
        let mut nodes: Vec<Token> = Vec::new();
        self.consume(TokenType::Lparen)?;

        if self.check(TokenType::Id) {
            nodes.push(self.consume(TokenType::Id)?);
            while !self.is_at_end() && self.check(TokenType::Comma) {
                self.consume(TokenType::Comma)?;
                nodes.push(self.consume(TokenType::Id)?);
            }
        }

        self.consume(TokenType::Rparen)?;
        Ok(nodes)
    }

    fn to_full_name(&self, tk: Token) -> Token{
        Token::new(
            TokenType::Text,
            format!("{:?}:{:?}", self.full_name_prefix, tk.value.clone()),
            tk.address.clone(),
        )
    }

    fn object_creation(&mut self) -> Result<Node, Error> {
        self.consume(TokenType::New)?;
        let name = self.consume(TokenType::Id)?;
        let args = self.args()?;
        Ok(Node::Instance {
            name,
            constructor: args,
            should_push: true
        })
    }

    fn access_part(&mut self, previous: Option<Box<Node>>) -> Result<Node, Error> {
        if self.check(TokenType::Id) {
            let identifier = self.consume(TokenType::Id)?;
            if self.check(TokenType::Walrus) {
                self.consume(TokenType::Walrus)?;
                Ok(Node::Define {
                    previous,
                    name: identifier,
                    value: Box::new(self.expr()?),
                })
            } else if self.check(TokenType::Assign) {
                self.consume(TokenType::Assign)?;
                Ok(Node::Assign {
                    previous,
                    name: identifier,
                    value: Box::new(self.expr()?),
                })
            } else if self.check(TokenType::AssignAdd) ||
                self.check(TokenType::AssignSub) ||
                self.check(TokenType::AssignMul) ||
                self.check(TokenType::AssignDiv) {
                let op;
                let loc;

                match self.peek()?.tk_type {
                    TokenType::AssignSub => {
                        loc = self.consume(TokenType::AssignSub)?;
                        op = "-".to_string();
                    }
                    TokenType::AssignMul => {
                        loc = self.consume(TokenType::AssignMul)?;
                        op = "*".to_string();
                    }
                    TokenType::AssignDiv => {
                        loc = self.consume(TokenType::AssignDiv)?;
                        op = "/".to_string();
                    }
                    TokenType::AssignAdd => {
                        loc = self.consume(TokenType::AssignAdd)?;
                        op = "+".to_string();
                    }
                    _ => {
                        panic!("invalid AssignOp tk_type. report to developer.");
                    }
                }
                let var = Node::Get {
                    previous: previous.clone(),
                    name: identifier.clone(),
                    should_push: true
                };
                return Ok(Node::Assign {
                    previous: previous.clone(),
                    name: identifier.clone(),
                    value: Box::new(Node::Bin {
                        left: Box::new(var),
                        right: Box::new(self.expr()?),
                        op: Token::new(
                            TokenType::Op,
                            op.clone(),
                            loc.address.clone(),
                        )
                    }),
                });
            } else if self.check(TokenType::Lparen) {
                return Ok(Node::Call {
                    previous: previous.clone(),
                    name: identifier.clone(),
                    args: self.args()?,
                    should_push: true
                });
            } else {
                return Ok(Node::Get {
                    previous: previous.clone(),
                    name: identifier.clone(),
                    should_push: true
                })
            }
        } else {
            Ok(self.object_creation()?)
        }
    }

    fn access(&mut self) -> Result<Node, Error> {
        let mut left = self.access_part(Option::None)?;

        while self.check(TokenType::Dot) {
            self.consume(TokenType::Dot)?;
            let location = self.peek()?.address.clone();
            left = self.access_part(Option::Some(Box::new(left)))?;
            match left {
                Node::Define { .. } => {
                    return Err(Error::new(
                        ErrorType::Parsing,
                        location,
                        "couldn't use define in expr.".to_string(),
                        "check your code.".to_string(),
                    ))
                }
                Node::Assign { .. } => {
                    return Err(Error::new(
                        ErrorType::Parsing,
                        location,
                        "couldn't use assign in expr.".to_string(),
                        "check your code.".to_string(),
                    ))
                }
                _ => {}
            }
        }

        Ok(left);
    }

    fn access_expr(&mut self) -> Result<Node, Error> {
        Ok(self.access()?)
    }

    fn access_statement(&mut self) -> Result<Node, Error> {
        let location = self.peek()?.address.clone();
        let result = set_should_push(self.access()?, true, location)?;
        Ok(result)
    }

    fn grouping(&mut self) -> Result<Node, Error> {
        self.consume(TokenType::Lparen)?;
        let expr = self.expr()?;
        self.consume(TokenType::Rparen)?;
        Ok(expr)
    }

    fn anonymous_fn(&mut self) -> Result<Node, Error> {
        let location = self.consume(TokenType::Fun)?;

        let mut params: Vec<Token> = Vec::new();
        if self.check(TokenType::Lparen) {
            params = self.params()?;
        }
        self.consume(TokenType::Lbrace)?;
        let body = self.block()?;
        self.consume(TokenType::Rbrace)?;

        Ok(Node::AnFnDeclaration {
            location,
            params,
            body: Box::new(body)
        })
    }

    fn lambda_fn(&mut self) -> Result<Node, Error> {
        let location = self.consume(TokenType::Lambda)?;

        let mut params: Vec<Token> = Vec::new();
        if self.check(TokenType::Lparen) {
            params = self.params()?;
        }
        self.consume(TokenType::Arrow)?;
        let body = self.expr()?;

        Ok(Node::AnFnDeclaration {
            location,
            params,
            body: Box::new(body)
        })
    }

    fn primary(&mut self) -> Result<Node, Error> {
        match self.peek()?.tk_type {
            TokenType::Id | TokenType::New => {
                let location = self.peek()?;
                Ok(self.access_expr()?)
            }
            TokenType::Number => {
                Ok(Node::Number {
                    value: self.consume(TokenType::Number)?
                })
            }
            TokenType::Text => {
                Ok(Node::String {
                    value: self.consume(TokenType::Text)?
                })
            }
            TokenType::Bool => {
                Ok(Node::Bool {
                    value: self.consume(TokenType::Bool)?
                })
            }
            TokenType::Lparen => {
                Ok(self.grouping()?)
            }
            TokenType::Lbrace => {
                Ok(self.map()?)
            }
            TokenType::Lbracket => {
                Ok(self.list()?)
            }
            TokenType::Null => {
                Ok(Node::Null {
                    location: self.consume(TokenType::Null)?
                })
            }
            TokenType::Fun => {
                Ok(self.anonymous_fn()?)
            }
            TokenType::Lambda => {
                Ok(self.lambda_fn()?)
            }
            TokenType::Match => {
                Ok(self.match_expr()?)
            }
            _ => Err(Error::new(
                ErrorType::Parsing,
                self.peek()?.address,
                format!("invalid token. {:?}:{:?}", self.peek()?.tk_type, self.peek()?.value),
                "check your code.".to_string(),
            ))
        }
    }

    fn list(&mut self) -> Result<Node, Error> {
        let location = self.consume(TokenType::Lbracket)?;
        if self.check(TokenType::Rbracket) {
            self.consume(TokenType::Rbracket)?;
            Ok(
                Node::List {
                    location,
                    values: Box::new(Vec::new())
                }
            )
        } else {
            let mut nodes: Vec<Box<Node>> = Vec::new();
            nodes.push(Box::new(self.expr()?));
            while self.check(TokenType::Comma) {
                self.consume(TokenType::Comma)?;
                nodes.push(Box::new(self.expr()?));
            }
            Ok(Node::List {
                location,
                values: Box::new(nodes)
            })
        }
    }

    fn key_value(&mut self) -> Result<(Box<Node>, Box<Node>), Error> {
        let l = self.expr()?;
        self.consume(TokenType::Colon);
        let r = self.expr()?;
        Ok((Box::new(l), Box::new(r)))
    }

    fn map(&mut self) -> Result<Node, Error> {
        let location = self.consume(TokenType::Lbracket)?;
        if self.check(TokenType::Rbracket) {
            self.consume(TokenType::Rbracket)?;
            Ok(
                Node::Map {
                    location,
                    values: Box::new(Vec::new())
                }
            )
        } else {
            let mut nodes: Vec<(Box<Node>, Box<Node>)> = Vec::new();
            let key = self.key_value()?;
            nodes.push((key.0, key.1));
            while self.check(TokenType::Comma) {
                self.consume(TokenType::Comma)?;
                let key = self.key_value()?;
                nodes.push((key.0, key.1));
            }
            Ok(Node::Map {
                location,
                values: Box::new(nodes)
            })
        }
    }

    fn unary(&mut self) -> Result<Node, Error> {
        let tk = self.peek()?;
        let _minus = String::from("-");
        match tk {
            Token { tk_type: TokenType::Op, value: _minus, .. } => {
                self.consume(TokenType::Op);
                Ok(Node::Unary {
                    op: self.peek()?,
                    value: Box::new(self.primary()?)
                })
            }
            _ => {
                Ok(self.primary()?)
            }
        }
    }

    fn multiplicative(&mut self) -> Result<Node, Error> {
        let mut left = self.unary()?;

        while self.check(TokenType::Op) &&
            (self.peek()?.value == "*" || self.peek()?.value == "/") {
            let op = self.peek()?;
            let right = self.unary()?;
            left = Node::Bin {
                left: Box::new(left),
                right: Box::new(right),
                op
            }
        }

        Ok(left)
    }

    fn additive(&mut self) -> Result<Node, Error> {
        let mut left = self.multiplicative()?;

        while self.check(TokenType::Op) &&
            (self.peek()?.value == "+" || self.peek()?.value == "-") {
            let op = self.peek()?;
            let right = self.multiplicative()?;
            left = Node::Bin {
                left: Box::new(left),
                right: Box::new(right),
                op
            }
        }

        Ok(left)
    }

    fn conditional(&mut self) -> Result<Node, Error> {
        let mut left = self.additive()?;

        if self.check(TokenType::Greater) || self.check(TokenType::Less)
            || self.check(TokenType::LessEq) || self.check(TokenType::GreaterEq) {
            let op = self.peek()?;
            let right = self.additive()?;
            left = Node::Cond {
                left: Box::new(left),
                right: Box::new(right),
                op
            };
        }

        Ok(left)
    }

    fn logical(&mut self) -> Result<Node, Error> {
        let mut left = self.conditional()?;

        while self.check(TokenType::And) ||
            self.check(TokenType::Or) {
            let op = self.peek()?;
            let right = self.conditional()?;
            left = Node::Logical {
                left: Box::new(left),
                right: Box::new(right),
                op
            };
        }

        Ok(left)
    }

    fn expr(&mut self) -> Result<Node, Error> {
        self.logical()
    }

    fn native(&mut self) -> Result<Node, Error> {
        let name = self.consume(TokenType::Id)?;
        self.consume(TokenType::Arrow)?;
        Ok(Node::Native {
            name
        })
    }

    fn function(&mut self) -> Result<Node, Error> {
        self.consume(TokenType::Fun)?;
        let name = self.consume(TokenType::Id)?;

        let mut params: Vec<Token> = Vec::new();
        if self.check(TokenType::Lparen) {
            params = self.params()?;
        }
        self.consume(TokenType::Lbrace)?;
        let body = self.block()?;
        self.consume(TokenType::Rbrace)?;

        Ok(Node::FnDeclaration {
            name: name.clone(),
            full_name: Option::Some(
                self.to_full_name(name.clone()),
            ),
            params,
            body: Box::new(body)
        })
    }

    fn type_stmt(&mut self) -> Result<Node, Error> {
        self.consume(TokenType::Type)?;
        let name = self.consume(TokenType::Id)?;

        let mut constructor: Vec<Token> = Vec::new();
        if self.check(TokenType::Lparen) {
            constructor = self.params()?;
        }
        self.consume(TokenType::Lbrace)?;
        let body = Vec::new();
        while !self.is_at_end() && !self.check(TokenType::Rbrace) {
            let location = self.peek()?;
            let mut node = self.statement()?;
            match node {
                Node::FnDeclaration { name, params, body, .. } => {
                    node = Node::FnDeclaration {
                        name,
                        full_name: None,
                        params,
                        body
                    }
                }
                Node::Native { .. } |
                Node::Get { .. } |
                Node::Define { .. } |
                Node::Assign { .. } => {}
                _ => {
                    return Err(Error::new(
                        ErrorType::Parsing,
                        location.address.clone(),
                        format!("invalid node for type: {:?}:{:?}", location.tk_type.clone(), location.value.clone()),
                        "check your code.".to_string(),
                    ));
                }
            }
        }
        self.consume(TokenType::Rbrace)?;

        Ok(Node::Type {
            name: name.clone(),
            full_name: Some(self.to_full_name(name.clone())),
            constructor,
            body: Box::new(Node::Block {
                body
            })
        })
    }

    fn statement(&mut self) -> Result<Node, Error> {
        todo!()
    }

    fn parse(&mut self) -> Result<Node, Error> {
        self.block()
    }

    fn consume(&mut self, tk_type: TokenType) -> Result<Token, Error> {
        match self.tokens.get(self.current as usize) {
            Some(tk) => {
                self.current += 1;
                if tk.tk_type == tk_type {
                    Ok(tk.clone())
                } else {
                    Err(Error::new(
                        ErrorType::Parsing,
                        tk.address.clone(),
                        format!("unexpected token {:?}:{:?}", tk.tk_type, tk.value),
                        "check your code.".to_string()
                    ))
                }
            },
            None => {
                Err(Error::new(
                    ErrorType::Parsing,
                    Address::new(0, self.filename.clone()),
                    "unexpected eof".to_string(),
                    "check your code.".to_string()
                ))
            }
        }
    }

    fn check(&self, tk_type: TokenType) -> bool {
        match self.tokens.get(self.current as usize) {
            Some(tk) => {
                if tk.tk_type == tk_type {
                    true
                } else {
                    false
                }
            },
            None => {
                false
            }
        }
    }

    fn peek(&self) -> Result<Token, Error> {
        match self.tokens.get(self.current as usize) {
            Some(tk) => {
                Ok(tk.clone())
            },
            None => {
                Err(Error::new(
                    ErrorType::Parsing,
                    Address::new(0, self.filename.clone()),
                    "unexpected eof".to_string(),
                    "check your code.".to_string()
                ))
            }
        }
    }

    fn is_at_end(&self) -> bool {
        self.current as usize >= self.tokens.len()
    }
}