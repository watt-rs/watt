#![allow(non_upper_case_globals)]

use std::collections::HashMap;
use crate::address::Address;
use crate::errors::{Error, ErrorType};

/*
Тип токена
 */

#[derive(Debug)]
#[derive(Clone)]
pub enum TokenType {
    Fun,
    Op, // +, -, *, /
    Lparen, // (
    Rparen, // )
    Lbrace, // {
    Rbrace, // }
    Lambda, // lambda
    Walrus, // :=
    Eq, // ==
    NotEq, // !=
    Text, // 'text'
    Number, // 1234567890.0123456789
    Assign, // =
    Id, // variable id
    Comma, // ,
    Ret, // return
    If, // if
    Bool, // bool
    While, // while
    Type, // type
    New, // new
    Dot, // dot
    Greater, // >
    Less,  // <
    GreaterEq, // >=
    LessEq, // <=
    Null, // null
    Elif, // elif
    Else, // else
    And, // logical and
    Or, // logical or
    Import, // import
    AssignAdd, // assign add
    AssignSub, // assign sub
    AssignMul, // assign mul
    AssignDiv,  // assign divide
    Break, // break
    Match, // match
    Case, // case
    Default, // default
    Lbracket, // [
    Rbracket, // ]
    Colon, // colon :
    For, // for
    Bang, // ?
    In, // in
    Continue, // continue
    Arrow, // ->
    Unit, // unit
    To, // to
    From, // from
    Native, // native
    Pipe, // pipe
    With // with
}

/*
Токен
 */

#[derive(Debug)]
#[derive(Clone)]
pub struct Token {
    tk_type: TokenType,
    value: String,
    address: Address,
}

impl Token {
    pub fn new(tk_type: TokenType, value: String, address: Address) -> Token {
        Token { tk_type, value, address }
    }
}

/*
Лексер
 */

pub struct Lexer {
    line: u64,
    current: u128,
    code: String,
    filename: String,
    tokens: Vec<Token>,
    keywords: HashMap<String, TokenType>,
}

impl Lexer {
    pub fn new(code: String, filename: String) -> Lexer {
        let map = HashMap::from([
            (String::from("fun"), TokenType::Fun),
            (String::from("break"), TokenType::Break),
            (String::from("if"), TokenType::If),
            (String::from("elif"), TokenType::Elif),
            (String::from("else"), TokenType::Else),
            (String::from("and"), TokenType::And),
            (String::from("or"), TokenType::Or),
            (String::from("import"), TokenType::Import),
            (String::from("type"), TokenType::Type),
            (String::from("new"), TokenType::New),
            (String::from("match"), TokenType::Match),
            (String::from("case"), TokenType::Case),
            (String::from("default"), TokenType::Default),
            (String::from("lambda"), TokenType::Lambda),
            (String::from("while"), TokenType::While),
            (String::from("unit"), TokenType::Unit),
            (String::from("for"), TokenType::For),
            (String::from("in"), TokenType::In),
            (String::from("continue"), TokenType::Continue),
            (String::from("from"), TokenType::From),
            (String::from("to"), TokenType::To),
        ]);
        Lexer {line: 1, current: 0, code, filename, tokens: vec![], keywords: map }
    }

    pub fn lex(&mut self) -> Result<Vec<Token>, Error> {
        while !self.is_at_end() {
            let ch = self.advance();
            match ch {
                '?' => { self.add_tk(TokenType::Bang, "?".to_string());  }
                '+' => {
                    if self.is_match('=') {
                        self.add_tk(TokenType::AssignAdd, "+=".to_string());
                    } else {
                        self.add_tk(TokenType::Op, "+".to_string());
                    }
                    
                }
                '-' => {
                    if self.is_match('=') {
                        self.add_tk(TokenType::AssignSub, "-=".to_string());
                    } else if self.is_match('>') {
                        self.add_tk(TokenType::Arrow, "->".to_string());
                    } else {
                        self.add_tk(TokenType::Op, "-".to_string());
                    }
                    
                }
                '*' => {
                    if self.is_match('=') {
                        self.add_tk(TokenType::AssignMul, "*=".to_string());
                    } else {
                        self.add_tk(TokenType::Op, "*".to_string());
                    }
                    
                }
                '/' => {
                    if self.is_match('=') {
                        self.add_tk(TokenType::AssignDiv, "/=".to_string());
                    } else if self.is_match('/') {
                        while !self.is_match('\n') && !self.is_at_end() {
                            self.advance();
                        }
                        self.line += 1;
                    } else if self.is_match('*') {
                        while !(self.peek() == '*' && self.next() == '/') && !self.is_at_end() {
                            if self.is_match('\n') {
                                self.line += 1;
                                continue;
                            }
                            self.advance();
                        }
                        // *
                        self.advance();
                        // /
                        self.advance();
                    } else {
                        self.add_tk(TokenType::Op, "/".to_string());
                    }
                    
                }
                '(' => {
                    self.add_tk(TokenType::Lparen, "(".to_string());
                    
                },
                ')' => {
                    self.add_tk(TokenType::Rparen, ")".to_string());
                    
                },
                '{' => {
                    self.add_tk(TokenType::Lbrace, "{".to_string());
                    
                },
                '}' => {
                    self.add_tk(TokenType::Rbrace, "}".to_string());
                    
                },
                '[' => {
                    self.add_tk(TokenType::Lbracket, "[".to_string());
                    
                },
                ']' => {
                    self.add_tk(TokenType::Rbracket, "]".to_string());
                    
                },
                ',' => {
                    self.add_tk(TokenType::Comma, ",".to_string());
                    
                },
                '.' => {
                    self.add_tk(TokenType::Dot, ".".to_string());
                    
                },
                ':' => {
                    if self.is_match('=') {
                        self.add_tk(TokenType::Walrus, ":=".to_string());
                    } else {
                        self.add_tk(TokenType::Colon, ":".to_string())
                    }
                    
                },
                '<' => {
                    if self.is_match('=') {
                        self.add_tk(TokenType::LessEq, "<=".to_string());
                    } else {
                        self.add_tk(TokenType::Less, "<".to_string());
                    }
                    
                }
                '>' => {
                    if self.is_match('=') {
                        self.add_tk(TokenType::GreaterEq, ">=".to_string());
                    } else {
                        self.add_tk(TokenType::Greater, ">".to_string());
                    }
                    
                }
                '!' => {
                    if self.is_match('=') {
                        self.add_tk(TokenType::NotEq, "!=".to_string());
                    } else {
                        self.add_tk(TokenType::Bang, "!".to_string());
                    }
                    
                }
                '=' => {
                    if self.is_match('=') {
                        self.add_tk(TokenType::Assign, "==".to_string());
                    } else {
                        self.add_tk(TokenType::Eq, "=".to_string());
                    }
                    
                }
                // пробелы
                '\r' => {
                    
                }
                '\t' => {
                    
                }
                '\n' => {
                    self.line += 1;
                    
                }
                ' ' => {
                    
                }
                '\'' => {
                    match self.scan_string() {
                        Ok(tk) => {
                            self.tokens.push(tk);
                        }
                        Err(err) => {
                            return Err(err);
                        }
                    }
                    
                }
                '|' => {
                    if self.is_match('>') {
                        self.add_tk(TokenType::Pipe, "|>".to_string());
                    } else {
                        return Err(Error::new(
                            ErrorType::Parsing,
                            Address::new(self.line, self.filename.clone()),
                            "expected > after | for pipe.".to_string(),
                            "check your code.".to_string()
                        ));
                    }
                    
                }
                _ => {
                    if self.is_digit(ch) {
                        match self.scan_number(ch) {
                            Ok(tk) => {
                                self.tokens.push(tk);
                            }
                            Err(err) => {
                                return Err(err);
                            }
                        }
                    }
                    else if self.is_id(ch) {
                        let token = self.scan_id_or_keyword(ch);
                        self.tokens.push(token);
                    }
                    else {
                        return Err(Error::new(
                            ErrorType::Parsing,
                            Address::new(self.line, self.filename.clone()),
                            format!("unexpected char: {}", ch),
                            format!("delete char: {}", ch),
                        ));
                    }
                    
                }
            }
        }
        Ok(self.tokens.clone())
    }

    fn scan_string(&mut self) -> Result<Token, Error> {
        let mut text: String = String::new();
        while self.peek() != '\'' {
            text.push(self.advance());
            if self.is_at_end() || self.is_match('\n') {
                return Err(
                    Error::new(
                        ErrorType::Parsing,
                        Address::new(
                            self.line,
                            self.filename.clone(),
                        ),
                        "unclosed string.".to_string(),
                        "did you forget ' symbol?".to_string(),
                    )
                )
            }
        }
        self.advance();
        Ok(Token {
            tk_type: TokenType::Number,
            value: text,
            address: Address::new(self.line, self.filename.clone()),
        })
    }

    fn scan_number(&mut self, start: char) -> Result<Token, Error> {
        let mut text: String = String::from(start);
        let mut is_float: bool = false;
        while self.is_digit(self.peek()) {
            if self.is_match('.') {
                if is_float {
                    return Err(
                        Error::new(
                            ErrorType::Parsing,
                            Address::new(
                                self.line,
                                self.filename.clone(),
                            ),
                            "couldn't parse number with two dots".to_string(),
                            "check your code.".to_string(),
                        )
                    )
                }
                is_float = true;
                continue;
            }
            text.push(self.advance());
            if self.is_at_end() {
                
            }
        }
        Ok(Token {
            tk_type: TokenType::Number,
            value: text,
            address: Address::new(self.line, self.filename.clone()),
        })
    }

    fn scan_id_or_keyword(&mut self, start: char) -> Token {
        let mut text: String = String::from(start);
        while self.is_id(self.peek()) {
            if self.is_match('\n') {
                self.line += 1;
                continue;
            }
            text.push(self.advance());
            if self.is_at_end() {
                
            }
        }
        let tk_type: TokenType = match self.keywords.get(&text) {
            Some(tk_type) => tk_type.clone(),
            None => TokenType::Id
        };
        Token {
            tk_type,
            value: text,
            address: Address::new(self.line, self.filename.clone()),
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.code.len() as u128
    }


    fn char_at(&self, offset: u128) -> char {
        match self.code.chars().nth((self.current + offset) as usize) {
            Some(ch) => {
                ch
            }
            None => {
                '\0'
            }
        }
    }

    fn advance(&mut self) -> char {
        let ch: char = self.char_at(0);
        self.current += 1;
        ch
    }

    fn peek(&self) -> char {
        if self.is_at_end() { '\0'; }
        self.char_at(0)
    }

    fn next(&self) -> char {
        if self.current + 1 >= self.code.len() as u128 {
            '\0'
        } else {
            self.char_at(1)
        }
    }

    fn is_match(&self, ch: char) -> bool {
        if self.is_at_end() {
            false
        }
        else {
            if self.char_at(self.current) == ch {
                true
            } else {
                false
            }
        }
    }

    fn add_tk(&mut self, tk_type: TokenType, tk_value: String) {
        self.tokens.push(
            Token::new(
                tk_type, tk_value, Address::new(self.line, self.filename.clone())
            )
        );
    }

    fn is_digit(&self, ch: char) -> bool {
        ch >= '0' && ch <= '9'
    }

    fn is_letter(&self, ch: char) -> bool {
        (ch >= 'a' && ch <= 'z') ||
        (ch >= 'A' && ch <= 'Z') ||
        (ch == '_')
    }

    fn is_id(&self, ch: char) -> bool {
        self.is_letter(ch) || self.is_digit(ch) ||
        (ch == ':' && self.is_id(self.next()))
    }
}