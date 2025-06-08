#![allow(unused_qualifications)]

use crate::lexer::address::*;
use crate::errors::{Error, ErrorType};
use crate::import::Import;
use crate::lexer::lexer::*;
use crate::parser::ast::*;

pub struct Parser {
    tokens: Vec<Token>,
    current: u128,
    filename: String,
    full_name_prefix: String,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, filename: String, full_name_prefix: String) -> Parser {
        Parser { tokens, current: 0, filename, full_name_prefix }
    }

    fn block(&mut self) -> Result<Node, Error> {
        let mut nodes: Vec<Box<Node>> = Vec::new();

        while !self.is_at_end() && !self.check(TokenType::Rbrace) {
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

    fn object_creation_expr(&mut self) -> Result<Node, Error> {
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
                            op,
                            loc.address,
                        )
                    }),
                });
            } else if self.check(TokenType::Lparen) {
                return Ok(Node::Call {
                    previous,
                    name: identifier.clone(),
                    args: self.args()?,
                    should_push: true
                });
            } else {
                return Ok(Node::Get {
                    previous,
                    name: identifier.clone(),
                    should_push: true
                })
            }
        } else {
            Ok(self.object_creation_expr()?)
        }
    }

    fn parse_access(&mut self, is_expr: bool) -> Result<Node, Error> {
        let mut left = self.access_part(Option::None)?;

        while self.check(TokenType::Dot) {
            self.consume(TokenType::Dot)?;
            let location = self.peek()?.address;
            left = self.access_part(Option::Some(Box::new(left)))?;
            if !is_expr { continue; }
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

        Ok(left)
    }

    fn access_expr(&mut self) -> Result<Node, Error> {
        Ok(self.parse_access(true)?)
    }

    fn access_stmt(&mut self) -> Result<Node, Error> {
        let location = self.peek()?.address;
        let result = set_should_push(self.parse_access(false)?, false, location)?;
        Ok(result)
    }

    fn grouping_expr(&mut self) -> Result<Node, Error> {
        self.consume(TokenType::Lparen)?;
        let expr = self.expr()?;
        self.consume(TokenType::Rparen)?;
        Ok(expr)
    }

    fn anonymous_fn_expr(&mut self) -> Result<Node, Error> {
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

    fn lambda_fn_expr(&mut self) -> Result<Node, Error> {
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

    fn primary_expr(&mut self) -> Result<Node, Error> {
        match self.peek()?.tk_type {
            TokenType::Id | TokenType::New => {
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
                Ok(self.grouping_expr()?)
            }
            TokenType::Lbrace => {
                Ok(self.map_expr()?)
            }
            TokenType::Lbracket => {
                Ok(self.list_expr()?)
            }
            TokenType::Null => {
                Ok(Node::Null {
                    location: self.consume(TokenType::Null)?
                })
            }
            TokenType::Fun => {
                Ok(self.anonymous_fn_expr()?)
            }
            TokenType::Lambda => {
                Ok(self.lambda_fn_expr()?)
            }
            // TokenType::Match => {
            //     Ok(self.match_expr()?)
            // }
            _ => Err(Error::new(
                ErrorType::Parsing,
                self.peek()?.address,
                format!("invalid token. {:?}:{:?}", self.peek()?.tk_type, self.peek()?.value),
                "check your code.".to_string(),
            ))
        }
    }

    fn list_expr(&mut self) -> Result<Node, Error> {
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

    fn key_value_expr(&mut self) -> Result<(Box<Node>, Box<Node>), Error> {
        let l = self.expr()?;
        self.consume(TokenType::Colon)?;
        let r = self.expr()?;
        Ok((Box::new(l), Box::new(r)))
    }

    fn map_expr(&mut self) -> Result<Node, Error> {
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
            let key = self.key_value_expr()?;
            nodes.push((key.0, key.1));
            while self.check(TokenType::Comma) {
                self.consume(TokenType::Comma)?;
                let key = self.key_value_expr()?;
                nodes.push((key.0, key.1));
            }
            Ok(Node::Map {
                location,
                values: Box::new(nodes)
            })
        }
    }

    fn unary_expr(&mut self) -> Result<Node, Error> {
        let tk = self.peek()?;
        match tk {
            Token { tk_type: TokenType::Op, value, .. } if value == "-" || value == "!"  => {
                let op = self.consume(TokenType::Op)?;
                Ok(Node::Unary {
                    op,
                    value: Box::new(self.primary_expr()?)
                })
            }
            _ => {
                Ok(self.primary_expr()?)
            }
        }
    }

    fn multiplicative_expr(&mut self) -> Result<Node, Error> {
        let mut left = self.unary_expr()?;

        while self.check(TokenType::Op) &&
            (self.peek()?.value == "*" || self.peek()?.value == "/") {
            let op = self.consume(TokenType::Op)?;
            let right = self.unary_expr()?;
            left = Node::Bin {
                left: Box::new(left),
                right: Box::new(right),
                op
            }
        }

        Ok(left)
    }

    fn additive_expr(&mut self) -> Result<Node, Error> {
        let mut left = self.multiplicative_expr()?;

        while self.check(TokenType::Op) &&
            (self.peek()?.value == "+" || self.peek()?.value == "-") {
            let op = self.consume(TokenType::Op)?;
            let right = self.multiplicative_expr()?;
            left = Node::Bin {
                left: Box::new(left),
                right: Box::new(right),
                op
            }
        }

        Ok(left)
    }

    fn conditional_expr(&mut self) -> Result<Node, Error> {
        let mut left = self.additive_expr()?;

        if self.check(TokenType::Greater) || self.check(TokenType::Less)
            || self.check(TokenType::LessEq) || self.check(TokenType::GreaterEq) ||
            self.check(TokenType::Eq) || self.check(TokenType::NotEq) {
            let op = self.peek()?;
            self.current += 1;
            let right = self.additive_expr()?;
            left = Node::Cond {
                left: Box::new(left),
                right: Box::new(right),
                op
            };
        }

        Ok(left)
    }

    fn logical_expr(&mut self) -> Result<Node, Error> {
        let mut left = self.conditional_expr()?;

        while self.check(TokenType::And) ||
            self.check(TokenType::Or) {
            let op = self.peek()?;
            self.current += 1;
            let right = self.conditional_expr()?;
            left = Node::Logical {
                left: Box::new(left),
                right: Box::new(right),
                op
            };
        }

        Ok(left)
    }

    fn expr(&mut self) -> Result<Node, Error> {
        self.logical_expr()
    }

    fn continue_stmt(&mut self) -> Result<Node, Error> {
        let location = self.consume(TokenType::Continue)?;
        Ok(Node::Continue {
            location
        })
    }

    fn break_stmt(&mut self) -> Result<Node, Error> {
        let location = self.consume(TokenType::Break)?;
        Ok(Node::Break {
            location
        })
    }

    fn return_stmt(&mut self) -> Result<Node, Error> {
        let location = self.consume(TokenType::Ret)?;
        let value = Box::new(self.expr()?);
        Ok(Node::Ret {
            location,
            value
        })
    }

    fn single_import(&mut self) -> Result<Import, Error> {
        let name = self.consume(TokenType::Text)?;
        if self.check(TokenType::With) {
            self.consume(TokenType::With)?;
            Ok(Import::new(
                name.value,
                Some(
                    self.consume(TokenType::Text)?.value
                )
            ))
        } else {
            Ok(Import::new(
                name.value,
                None
            ))
        }
    }

    fn import_stmt(&mut self) -> Result<Node, Error> {
        self.consume(TokenType::Import)?;
        let mut imports = Vec::new();
        if self.check(TokenType::Lparen) {
            self.consume(TokenType::Lparen)?;
            imports.push(self.single_import()?);
            while self.check(TokenType::Comma) {
                self.consume(TokenType::Comma)?;
                imports.push(self.single_import()?);
            }
        }
        else {
            imports.push(self.single_import()?);
        }
        Ok(Node::Import {
            imports
        })
    }

    fn while_stmt(&mut self) -> Result<Node, Error> {
        let location = self.consume(TokenType::While)?;
        let logical = self.expr()?;
        self.consume(TokenType::Lbrace)?;
        let body = self.block()?;
        self.consume(TokenType::Rbrace)?;
        Ok(Node::While {
            location,
            logical: Box::new(logical),
            body: Box::new(body)
        })
    }

    fn else_stmt(&mut self) -> Result<Node, Error> {
        let location = self.consume(TokenType::Else)?;
        self.consume(TokenType::Lbrace)?;
        let body = self.block()?;
        self.consume(TokenType::Rbrace)?;
        Ok(Node::If {
            location: location.clone(),
            logical: Box::new(Node::Bool { value: Token::new(
                TokenType::Bool,
                "true".to_string(),
                location.address
            )}),
            body: Box::new(body),
            elseif: None
        })
    }

    //noinspection ALL
    fn elif_stmt(&mut self) -> Result<Node, Error> {
        let location = self.consume(TokenType::Elif)?;
        let logical = self.expr()?;
        self.consume(TokenType::Lbrace)?;
        let body = self.block()?;
        self.consume(TokenType::Rbrace)?;
        if self.check(TokenType::Elif) {
            Ok(Node::If {
                location,
                logical: Box::new(logical),
                body: Box::new(body),
                elseif: Some(Box::new(self.elif_stmt()?))
            })
        } else if self.check(TokenType::Else) {
            Ok(Node::If {
                location,
                logical: Box::new(logical),
                body: Box::new(body),
                elseif: Some(Box::new(self.else_stmt()?))
            })
        } else {
            Ok(Node::If {
                location,
                logical: Box::new(logical),
                body: Box::new(body),
                elseif: None
            })
        }
    }

    //noinspection ALL
    fn if_stmt(&mut self) -> Result<Node, Error> {
        let location = self.consume(TokenType::If)?;
        let logical = self.expr()?;
        self.consume(TokenType::Lbrace)?;
        let body = self.block()?;
        self.consume(TokenType::Rbrace)?;
        if self.check(TokenType::Elif) {
            Ok(Node::If {
                location,
                logical: Box::new(logical),
                body: Box::new(body),
                elseif: Some(Box::new(self.elif_stmt()?))
            })
        } else if self.check(TokenType::Else) {
            Ok(Node::If {
                location,
                logical: Box::new(logical),
                body: Box::new(body),
                elseif: Some(Box::new(self.else_stmt()?))
            })
        } else {
            Ok(Node::If {
                location,
                logical: Box::new(logical),
                body: Box::new(body),
                elseif: None
            })
        }
    }

    fn for_stmt(&mut self) -> Result<Node, Error> {
        self.consume(TokenType::For)?;
        let name = self.consume(TokenType::Id)?;
        self.consume(TokenType::In)?;
        let value = self.expr()?;
        self.consume(TokenType::Lbrace)?;
        let body = self.block()?;
        self.consume(TokenType::Rbrace)?;
        Ok(Node::For {
            variable_name: name,
            iterable: Box::new(value),
            body: Box::new(body),
        })
    }

    fn function_stmt(&mut self) -> Result<Node, Error> {
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
                self.to_full_name(name),
            ),
            params,
            body: Box::new(body)
        })
    }

    //noinspection ALL
    fn type_stmt(&mut self) -> Result<Node, Error> {
        self.consume(TokenType::Type)?;
        let name = self.consume(TokenType::Id)?;

        let mut constructor: Vec<Token> = Vec::new();
        if self.check(TokenType::Lparen) {
            constructor = self.params()?;
        }
        self.consume(TokenType::Lbrace)?;
        let mut body = Vec::new();
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
                        location.address,
                        format!("invalid node for type: {:?}:{:?}", location.tk_type, location.value),
                        "check your code.".to_string(),
                    ));
                }
            }
            body.push(Box::new(node));
        }
        self.consume(TokenType::Rbrace)?;

        Ok(Node::Type {
            name: name.clone(),
            full_name: Some(self.to_full_name(name)),
            constructor,
            body: Box::new(Node::Block {
                body
            })
        })
    }

    //noinspection ALL
    fn unit_stmt(&mut self) -> Result<Node, Error> {
        self.consume(TokenType::Unit)?;
        let name = self.consume(TokenType::Id)?;

        self.consume(TokenType::Lbrace)?;
        let mut body = Vec::new();
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
                        location.address,
                        format!("invalid node for unit: {:?}:{:?}", location.tk_type, location.value),
                        "check your code.".to_string(),
                    ));
                }
            }
            body.push(Box::new(node));
        }
        self.consume(TokenType::Rbrace)?;

        Ok(Node::Unit {
            name: name.clone(),
            full_name: Some(self.to_full_name(name)),
            body: Box::new(Node::Block {
                body
            })
        })
    }

    fn native_stmt(&mut self) -> Result<Node, Error> {
        self.consume(TokenType::Native)?;
        let name = self.consume(TokenType::Id)?;
        self.consume(TokenType::Arrow)?;
        let fn_name = self.consume(TokenType::Text)?;
        Ok(Node::Native {
            name,
            fn_name
        })
    }

    fn statement(&mut self) -> Result<Node, Error> {
        let tk = self.peek()?;
        match tk.tk_type {
            TokenType::Type => {
                self.type_stmt()
            },
            TokenType::Unit => {
                self.unit_stmt()
            },
            TokenType::If => {
                self.if_stmt()
            },
            TokenType::New | TokenType::Id => {
                self.access_stmt()
            },
            TokenType::Match => {
                todo!()
            },
            TokenType::Continue => {
                self.continue_stmt()
            },
            TokenType::Break => {
                self.break_stmt()
            },
            TokenType::Ret => {
                self.return_stmt()
            },
            TokenType::Fun => {
                self.function_stmt()
            },
            TokenType::Native => {
                self.native_stmt()
            },
            TokenType::Import => {
                self.import_stmt()
            }
            TokenType::For => {
                self.for_stmt()
            }
            TokenType::While => {
                self.while_stmt()
            }
            _ => {
                Err(Error::new(
                    ErrorType::Parsing,
                    tk.address,
                    format!("unexpected token: {:?}:{tk_value}",
                            tk.tk_type, tk_value=tk.value),
                    "check your code.".to_string(),
                ))
            }
        }
    }

    pub fn parse(&mut self) -> Result<Node, Error> {
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
                        format!("unexpected token: {:?}:{:?}", tk.tk_type, tk.value),
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