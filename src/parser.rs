use std::io::BufRead;
use crate::address::Address;
use crate::ast::Node;
use crate::errors::{Error, ErrorType};
use crate::lexer::{Token, TokenType};

struct Parser {
    tokens: Vec<Token>,
    current: u128,
    filename: String,
}

impl Parser {
    fn new(filename: String) -> Parser {
        Parser { tokens: vec![], current: 0, filename }
    }

    fn block(&mut self) -> Result<Node, Error> {
        self.consume(TokenType::Lbrace).map_err(|e| e)?;
        // ...
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
                    String::from("check your code.")
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

    fn peek(&self) -> Option<Token> {
        match self.tokens.get(self.current as usize) {
            Some(tk) => {
                Some(tk.clone())
            },
            None => {
                None
            }
        }
    }
}