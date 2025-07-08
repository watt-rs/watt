// импорты
use crate::error;
use crate::errors::errors::Error;
use crate::lexer::address::*;
use std::collections::HashMap;
use crate::lexer::cursor::Cursor;

// тип токена
#[derive(Debug, Clone, Eq, PartialEq, Copy, Hash)]
#[allow(dead_code)]
pub enum TokenType {
    Fun,
    Op,        // +, -, *, /
    Lparen,    // (
    Rparen,    // )
    Lbrace,    // {
    Rbrace,    // }
    Lambda,    // lambda
    Walrus,    // :=
    Eq,        // ==
    NotEq,     // !=
    Text,      // 'text'
    Number,    // 1234567890.0123456789
    Assign,    // =
    Id,        // variable id
    Comma,     // ,
    Ret,       // return
    If,        // if
    Bool,      // bool
    While,     // while
    Type,      // type
    New,       // new
    Dot,       // dot
    Greater,   // >
    Less,      // <
    GreaterEq, // >=
    LessEq,    // <=
    Null,      // null
    Elif,      // elif
    Else,      // else
    And,       // logical and
    Or,        // logical or
    Import,    // import
    AssignAdd, // assign add
    AssignSub, // assign sub
    AssignMul, // assign mul
    AssignDiv, // assign divide
    Break,     // break
    Match,     // match
    Case,      // case
    Default,   // default
    Lbracket,  // [
    Rbracket,  // ]
    Colon,     // colon :
    For,       // for
    Bang,      // !
    In,        // in
    Continue,  // continue
    Arrow,     // ->
    Unit,      // unit
    Native,    // native
    With,      // with
    Trait,     // trait
    Impl,      // impl
    Question,  // ?
    Impls,     // impls
    Range,     // ..
}

// токен
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Token {
    pub tk_type: TokenType,
    pub value: String,
    pub address: Address,
}
// имплементация
impl Token {
    pub fn new(tk_type: TokenType, value: String, address: Address) -> Token {
        Token {
            tk_type,
            value,
            address,
        }
    }
}

// лексер
pub struct Lexer<'filename, 'cursor> {
    line: u64,
    column: u16,
    cursor: Cursor<'cursor>,
    line_text: String,
    filename: &'filename str,
    tokens: Vec<Token>,
    keywords: HashMap<&'static str, TokenType>,
}

// имплементация
impl<'filename, 'cursor> Lexer<'filename, 'cursor> {
    pub fn new(code: &'cursor [char], filename: &'filename str) -> Self {
        let map = HashMap::from([
            ("fun", TokenType::Fun),
            ("break", TokenType::Break),
            ("if", TokenType::If),
            ("elif", TokenType::Elif),
            ("else", TokenType::Else),
            ("and", TokenType::And),
            ("or", TokenType::Or),
            ("import", TokenType::Import),
            ("type", TokenType::Type),
            ("new", TokenType::New),
            ("match", TokenType::Match),
            ("case", TokenType::Case),
            ("default", TokenType::Default),
            ("lambda", TokenType::Lambda),
            ("while", TokenType::While),
            ("unit", TokenType::Unit),
            ("for", TokenType::For),
            ("in", TokenType::In),
            ("continue", TokenType::Continue),
            ("true", TokenType::Bool),
            ("false", TokenType::Bool),
            ("null", TokenType::Null),
            ("return", TokenType::Ret),
            ("trait", TokenType::Trait),
            ("impl", TokenType::Impl),
            ("native", TokenType::Native),
            ("impls", TokenType::Impls),
        ]);
        // лексер
        let mut lexer = Lexer {
            line: 1,
            column: 0,
            line_text: String::new(),
            cursor: Cursor::new(code),
            filename,
            tokens: vec![],
            keywords: map,
        };
        // текст первой линии
        lexer.line_text = lexer.get_line_text();
        // возвращаем
        lexer
    }

    pub fn lex(mut self) -> Vec<Token> {
        if self.tokens.len() > 0 {
            panic!("tokens len already > 0. report this error to the developer.")
        }
        while !self.cursor.is_at_end() {
            let ch = self.advance();
            match ch {
                '+' => {
                    if self.is_match('=') {
                        self.add_tk(TokenType::AssignAdd, "+=");
                    } else {
                        self.add_tk(TokenType::Op, "+");
                    }
                }
                '-' => {
                    if self.is_match('=') {
                        self.add_tk(TokenType::AssignSub, "-=");
                    } else if self.is_match('>') {
                        self.add_tk(TokenType::Arrow, "->");
                    } else {
                        self.add_tk(TokenType::Op, "-");
                    }
                }
                '*' => {
                    if self.is_match('=') {
                        self.add_tk(TokenType::AssignMul, "*=");
                    } else {
                        self.add_tk(TokenType::Op, "*");
                    }
                }
                '%' => {
                    self.add_tk(TokenType::Op, "%");
                }
                '/' => {
                    if self.is_match('=') {
                        self.add_tk(TokenType::AssignDiv, "/=");
                    } else if self.is_match('/') {
                        while !self.is_match('\n') && !self.cursor.is_at_end() {
                            self.advance();
                        }
                        self.newline();
                    } else if self.is_match('*') {
                        while !(self.cursor.peek() == '*' && self.cursor.next() == '/') && !self.cursor.is_at_end() {
                            if self.is_match('\n') {
                                self.newline();
                                continue;
                            }
                            self.advance();
                        }
                        // *
                        self.advance();
                        // /
                        self.advance();
                    } else {
                        self.add_tk(TokenType::Op, "/");
                    }
                }
                '(' => {
                    self.add_tk(TokenType::Lparen, "(");
                }
                ')' => {
                    self.add_tk(TokenType::Rparen, ")");
                }
                '{' => {
                    self.add_tk(TokenType::Lbrace, "{");
                }
                '}' => {
                    self.add_tk(TokenType::Rbrace, "}");
                }
                '[' => {
                    self.add_tk(TokenType::Lbracket, "[");
                }
                ']' => {
                    self.add_tk(TokenType::Rbracket, "]");
                }
                ',' => {
                    self.add_tk(TokenType::Comma, ",");
                }
                '.' => {
                    if self.is_match('.') {
                        self.add_tk(TokenType::Range, "..");
                    } else {
                        self.add_tk(TokenType::Dot, ".");
                    }
                }
                '?' => {
                    self.add_tk(TokenType::Question, "?");
                }
                ':' => {
                    if self.is_match('=') {
                        self.add_tk(TokenType::Walrus, ":=");
                    } else {
                        self.add_tk(TokenType::Colon, ":")
                    }
                }
                '<' => {
                    if self.is_match('=') {
                        self.add_tk(TokenType::LessEq, "<=");
                    } else {
                        self.add_tk(TokenType::Less, "<");
                    }
                }
                '>' => {
                    if self.is_match('=') {
                        self.add_tk(TokenType::GreaterEq, ">=");
                    } else {
                        self.add_tk(TokenType::Greater, ">");
                    }
                }
                '!' => {
                    if self.is_match('=') {
                        self.add_tk(TokenType::NotEq, "!=");
                    } else {
                        self.add_tk(TokenType::Bang, "!");
                    }
                }
                '=' => {
                    if self.is_match('=') {
                        self.add_tk(TokenType::Eq, "==");
                    } else {
                        self.add_tk(TokenType::Assign, "=");
                    }
                }
                // пробелы
                '\r' => {}
                '\t' => {}
                '\0' => {}
                '\n' => {
                    self.newline();
                }
                ' ' => {}
                '\'' => match self.scan_string() {
                    Ok(tk) => {
                        self.tokens.push(tk);
                    }
                    Err(err) => {
                        error!(err);
                    }
                },
                _ => {
                    if self.is_digit(ch) {
                        match self.scan_number(ch) {
                            Ok(tk) => {
                                self.tokens.push(tk);
                            }
                            Err(err) => {
                                error!(err);
                            }
                        }
                    } else if self.is_id(ch) {
                        let token = self.scan_id_or_keyword(ch);
                        self.tokens.push(token);
                    } else {
                        error!(Error::own(
                            Address::new(
                                self.line,
                                self.column,
                                self.filename.to_string(),
                                self.line_text.clone(),
                            ),
                            format!("unexpected char: {}", ch),
                            format!("delete char: {}", ch),
                        ));
                    }
                }
            }
        }
        self.tokens
    }

    fn scan_string(&mut self) -> Result<Token, Error> {
        let mut text: String = String::new();
        while self.cursor.peek() != '\'' {
            // символ
            let ch = self.advance();
            
            // если текущий символ "\", а следующий "'"
            if ch == '\\' && self.cursor.peek() == '\'' {
                text.push(self.advance());
            } else {
                text.push(ch);
            }

            // проверка на новую линию
            if self.cursor.is_at_end() || self.is_match('\n') {
                return Err(Error::new(
                    Address::new(
                        self.line,
                        self.column,
                        self.filename.to_string(),
                        self.line_text.clone(),
                    ),
                    "unclosed string quotes.",
                    "did you forget ' symbol?",
                ));
            }
        }

        self.advance();

        Ok(Token {
            tk_type: TokenType::Text,
            value: text,
            address: Address::new(
                self.line,
                self.column,
                self.filename.to_string(),
                self.line_text.clone(),
            ),
        })
    }

    fn scan_number(&mut self, start: char) -> Result<Token, Error> {
        let mut text: String = String::from(start);
        let mut is_float: bool = false;
        while self.is_digit(self.cursor.peek()) || self.cursor.peek() == '.' {
            if self.cursor.peek() == '.' {
                if self.cursor.next() == '.' {
                    break;
                }
                if is_float {
                    return Err(Error::new(
                        Address::new(
                            self.line,
                            self.column,
                            self.filename.to_string(),
                            self.line_text.clone(),
                        ),
                        "couldn't parse number with two dots",
                        "check your code.",
                    ));
                }
                is_float = true;
                text.push(self.advance());
                continue;
            }
            text.push(self.advance());
            if self.cursor.is_at_end() {
                break;
            }
        }
        Ok(Token {
            tk_type: TokenType::Number,
            value: text,
            address: Address::new(
                self.line,
                self.column,
                self.filename.to_string(),
                self.line_text.clone(),
            ),
        })
    }

    fn scan_id_or_keyword(&mut self, start: char) -> Token {
        let mut text: String = String::from(start);
        
        while self.is_id(self.cursor.peek()) {
            text.push(self.advance());
            if self.cursor.is_at_end() {
                break;
            }
        }
        
        let tk_type: TokenType = self.keywords.get(text.as_str()).cloned().unwrap_or(TokenType::Id);
        
        Token {
            tk_type,
            value: text,
            address: Address::new(
                self.line,
                self.column,
                self.filename.to_string(),
                self.line_text.clone(),
            ),
        }
    }


    fn get_line_text(&self) -> String {
        // проходимся по тексту
        let mut i = 0;
        let mut line_text = String::new();
        while !self.cursor.is_at_end_offset(i) && self.cursor.char_at(i) != '\n' {
            line_text.push(self.cursor.char_at(i));
            i += 1;
        }
        // возвращаем
        line_text
    }

    fn newline(&mut self) {
        self.line += 1;
        self.column = 0;
        self.line_text = self.get_line_text();
    }

    fn advance(&mut self) -> char {
        let ch: char = self.cursor.char_at(0);
        self.cursor.current += 1;
        self.column += 1;
        ch
    }


    #[allow(clippy::wrong_self_convention)]
    fn is_match(&mut self, ch: char) -> bool {
        if !self.cursor.is_at_end() {
            if self.cursor.char_at(0) == ch {
                self.advance();
                return true
            }
        }
        false
    }

    fn add_tk(&mut self, tk_type: TokenType, tk_value: &str) {
        self.tokens.push(Token::new(
            tk_type,
            tk_value.to_string(),
            Address::new(
                self.line,
                self.column,
                self.filename.to_string(),
                self.line_text.clone(),
            ),
        ));
    }

    fn is_digit(&self, ch: char) -> bool {
        ch >= '0' && ch <= '9'
    }

    fn is_letter(&self, ch: char) -> bool {
        (ch >= 'a' && ch <= 'z') || (ch >= 'A' && ch <= 'Z') || (ch == '_')
    }

    fn is_id(&self, ch: char) -> bool {
        self.is_letter(ch) || self.is_digit(ch) || (ch == ':' && self.is_id(self.cursor.next()))
    }
}
