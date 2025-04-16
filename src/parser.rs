use crate::address::Address;
use crate::ast::Node;
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

    fn access_expr(&mut self) -> Result<Node, Error> {
        let mut left = self.access_part(Option::None)?;

        while self.check(TokenType::Dot) {
            self.consume(TokenType::Dot)?;
            left = self.access_part(Option::Some(Box::new(left)))?;
            match left {
                Node::Define => {
                    Err(Error::new(

                    ))
                }
            }
        }

        Ok(left)
    }

    fn grouping(&mut self) -> Result<Node, Error> {
        self.consume(TokenType::Lparen)?;
        let expr = self.expr()?;
        self.consume(TokenType::Rparen)?;
        Ok(expr)
    }

    fn primary(&mut self) -> Result<Node, Error> {
        match self.peek()?.tk_type {
            TokenType::Id | TokenType::New => {
                let location = self.peek()?;
                let access = self.access_expr()?;
                if self.check(TokenType::Pipe) {
                    let mut pipe = self.pipe()?;
                    match pipe {
                        Node::Call { .. } => {
                            pipe = Node::Call {
                                previous: pipe.previous,
                                name: pipe.name,
                                args: pipe.args,
                                should_push: true,
                            }
                        }
                        _ => {
                            return Err(Error::new(
                                ErrorType::Parsing,
                                location.address,
                                "invalid pipe expr.".to_string(),
                                "call is one available expr in pipe.".to_string(),
                            ))
                        }
                    }
                    Ok(pipe)
                } else {
                    Ok(access)
                }
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
        return self.current as usize >= self.tokens.len();
    }
}