// imports
use crate::cursor::Cursor;
use crate::tokens::*;
use std::collections::HashMap;
use std::path::PathBuf;
use watt_common::{address::Address, error, errors::Error};

/// Lexer structure
pub struct Lexer<'file_path, 'cursor> {
    line: u64,
    column: u16,
    cursor: Cursor<'cursor>,
    file_path: &'file_path PathBuf,
    tokens: Vec<Token>,
    keywords: HashMap<&'static str, TokenKind>,
}
/// Lexer implementation
impl<'file_path, 'cursor> Lexer<'file_path, 'cursor> {
    /// Creates new lexer from
    ///
    /// * `code`: source code represented as `&'cursor [char]`
    /// * `file_path`: source file path
    ///
    pub fn new(code: &'cursor [char], file_path: &'file_path PathBuf) -> Self {
        // Keywords list
        let keywords_map = HashMap::from([
            ("fn", TokenKind::Fn),
            ("break", TokenKind::Break),
            ("if", TokenKind::If),
            ("elif", TokenKind::Elif),
            ("else", TokenKind::Else),
            ("and", TokenKind::And),
            ("or", TokenKind::Or),
            ("import", TokenKind::Import),
            ("type", TokenKind::Type),
            ("new", TokenKind::New),
            ("match", TokenKind::Match),
            ("case", TokenKind::Case),
            ("default", TokenKind::Default),
            ("lambda", TokenKind::Lambda),
            ("while", TokenKind::While),
            ("unit", TokenKind::Unit),
            ("for", TokenKind::For),
            ("in", TokenKind::In),
            ("continue", TokenKind::Continue),
            ("true", TokenKind::Bool),
            ("false", TokenKind::Bool),
            ("null", TokenKind::Null),
            ("return", TokenKind::Ret),
            ("trait", TokenKind::Trait),
            ("impl", TokenKind::Impl),
            ("native", TokenKind::Native),
            ("impls", TokenKind::Impls),
            ("as", TokenKind::As),
        ]);
        // Lexer
        Lexer {
            line: 1,
            column: 0,
            cursor: Cursor::new(code),
            file_path,
            tokens: vec![],
            keywords: keywords_map,
        }
    }

    /// Converts source code represented as `&'cursor [char]`
    /// To a `Vec<Token>` - tokens list.
    #[allow(clippy::nonminimal_bool)]
    pub fn lex(mut self) -> Vec<Token> {
        if !self.tokens.is_empty() {
            panic!("tokens len already > 0. report this error to the developer.")
        }
        while !self.cursor.is_at_end() {
            let ch = self.advance();
            match ch {
                '+' => {
                    if self.is_match('=') {
                        self.add_tk(TokenKind::AddEq, "+=");
                    } else {
                        self.add_tk(TokenKind::Plus, "+");
                    }
                }
                '&' => {
                    if self.is_match('=') {
                        self.add_tk(TokenKind::AndEq, "&=");
                    } else {
                        self.add_tk(TokenKind::BitwiseAnd, "&");
                    }
                }
                '|' => {
                    if self.is_match('=') {
                        self.add_tk(TokenKind::OrEq, "|=");
                    } else {
                        self.add_tk(TokenKind::BitwiseOr, "|");
                    }
                }
                '^' => {
                    if self.is_match('=') {
                        self.add_tk(TokenKind::XorEq, "^=");
                    } else {
                        self.add_tk(TokenKind::Caret, "^");
                    }
                }
                '-' => {
                    if self.is_match('=') {
                        self.add_tk(TokenKind::SubEq, "-=");
                    } else if self.is_match('>') {
                        self.add_tk(TokenKind::Arrow, "->");
                    } else {
                        self.add_tk(TokenKind::Minus, "-");
                    }
                }
                '*' => {
                    if self.is_match('=') {
                        self.add_tk(TokenKind::MulEq, "*=");
                    } else {
                        self.add_tk(TokenKind::Star, "*");
                    }
                }
                '%' => {
                    self.add_tk(TokenKind::Percent, "%");
                }
                '/' => {
                    // compound operator
                    if self.is_match('=') {
                        self.add_tk(TokenKind::DivEq, "/=");
                    }
                    // line comment
                    else if self.is_match('/') {
                        while !self.is_match('\n') && !self.cursor.is_at_end() {
                            self.advance();
                        }
                        self.new_line();
                    }
                    // multi-line comment
                    else if self.is_match('*') {
                        while !(self.cursor.peek() == '*' && self.cursor.next() == '/')
                            && !self.cursor.is_at_end()
                        {
                            if self.is_match('\n') {
                                self.new_line();
                                continue;
                            }
                            self.advance();
                        }
                        // *
                        self.advance();
                        // /
                        self.advance();
                    } else {
                        self.add_tk(TokenKind::Slash, "/");
                    }
                }
                '(' => {
                    self.add_tk(TokenKind::Lparen, "(");
                }
                ')' => {
                    self.add_tk(TokenKind::Rparen, ")");
                }
                '{' => {
                    self.add_tk(TokenKind::Lbrace, "{");
                }
                '}' => {
                    self.add_tk(TokenKind::Rbrace, "}");
                }
                '[' => {
                    self.add_tk(TokenKind::Lbracket, "[");
                }
                ']' => {
                    self.add_tk(TokenKind::Rbracket, "]");
                }
                ',' => {
                    self.add_tk(TokenKind::Comma, ",");
                }
                '.' => {
                    if self.is_match('.') {
                        self.add_tk(TokenKind::Range, "..");
                    } else {
                        self.add_tk(TokenKind::Dot, ".");
                    }
                }
                '?' => {
                    self.add_tk(TokenKind::Question, "?");
                }
                ':' => {
                    if self.is_match('=') {
                        self.add_tk(TokenKind::Walrus, ":=");
                    } else {
                        self.add_tk(TokenKind::Colon, ":")
                    }
                }
                '<' => {
                    if self.is_match('=') {
                        self.add_tk(TokenKind::LessEq, "<=");
                    } else {
                        self.add_tk(TokenKind::Less, "<");
                    }
                }
                '>' => {
                    if self.is_match('=') {
                        self.add_tk(TokenKind::GreaterEq, ">=");
                    } else {
                        self.add_tk(TokenKind::Greater, ">");
                    }
                }
                '!' => {
                    if self.is_match('=') {
                        self.add_tk(TokenKind::NotEq, "!=");
                    } else {
                        self.add_tk(TokenKind::Bang, "!");
                    }
                }
                '=' => {
                    if self.is_match('=') {
                        self.add_tk(TokenKind::Eq, "==");
                    } else {
                        self.add_tk(TokenKind::Assign, "=");
                    }
                }
                '\r' => {}
                '\t' => {}
                '\0' => {}
                ' ' => {}
                '\n' => {
                    self.new_line();
                }
                '\'' => {
                    let tk = self.scan_string();
                    self.tokens.push(tk)
                }
                _ => {
                    // numbers
                    if self.is_digit(ch) {
                        // different number types scanning
                        let tk;
                        if self.cursor.peek() == 'x' {
                            tk = self.scan_hexadecimal_number();
                        } else if self.cursor.peek() == 'o' {
                            tk = self.scan_octal_number();
                        } else if self.cursor.peek() == 'b' {
                            tk = self.scan_binary_number();
                        } else {
                            tk = self.scan_number(ch);
                        }
                        self.tokens.push(tk);
                    }
                    // identifier
                    else if self.is_id(ch) {
                        let token = self.scan_id_or_keyword(ch);
                        self.tokens.push(token);
                    }
                    // unexpected
                    else {
                        error!(Error::own(
                            Address::new(self.line, self.column, self.file_path.clone(),),
                            format!("unexpected char: {ch}"),
                            format!("delete char: {ch}"),
                        ));
                    }
                }
            }
        }
        self.tokens
    }

    /// Scans string. Implies quote is already ate. East ending quote.
    fn scan_string(&mut self) -> Token {
        // String text
        let mut text: String = String::new();
        let span_start = self.column;

        while self.cursor.peek() != '\'' {
            let ch = self.advance();

            if ch == '\\' && self.cursor.peek() == '\'' {
                text.push(self.advance());
            } else {
                text.push(ch);
            }

            if self.cursor.is_at_end() || self.is_match('\n') {
                error!(Error::new(
                    Address::new(self.line, self.column, self.file_path.clone(),),
                    "unclosed string quotes.",
                    "did you forget ' symbol?",
                ));
            }
        }

        self.advance();
        let span_end = self.column;

        Token {
            tk_type: TokenKind::Text,
            value: text,
            address: Address::span(self.line, span_start..span_end, self.file_path.clone()),
        }
    }

    /// Scans decimal and integer numbers
    ///
    /// # Arguments
    /// * `start`: starting char of token
    ///
    fn scan_number(&mut self, start: char) -> Token {
        // Start of span
        let span_start = self.column;
        // Number text
        let mut text: String = String::from(start);
        // If number is float
        let mut is_float: bool = false;

        while self.is_digit(self.cursor.peek()) || self.cursor.peek() == '.' {
            if self.cursor.peek() == '.' {
                if self.cursor.next() == '.' {
                    break;
                }
                if is_float {
                    error!(Error::new(
                        Address::new(self.line, self.column, self.file_path.clone(),),
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

        let span_end = self.column;

        Token {
            tk_type: TokenKind::Number,
            value: text,
            address: Address::span(self.line, span_start..span_end, self.file_path.clone()),
        }
    }

    /// Scans hexadecimal numbers `0x{pattern}`
    fn scan_hexadecimal_number(&mut self) -> Token {
        // Start of span
        let span_start = self.column;
        // Skip 'x'
        self.advance();
        // Number text
        let mut text: String = String::from("0x");

        while self.cursor.peek().is_ascii_hexdigit() {
            text.push(self.advance());
            if self.cursor.is_at_end() {
                break;
            }
        }

        let span_end = self.column;

        Token {
            tk_type: TokenKind::Number,
            value: text,
            address: Address::span(self.line, span_start..span_end, self.file_path.clone()),
        }
    }

    /// Scans octal numbers `0o{pattern}`
    fn scan_octal_number(&mut self) -> Token {
        // Start of span
        let span_start = self.column;
        // Skip 'o'
        self.advance();
        // Number text
        let mut text: String = String::from("0o");

        while self.cursor.peek().is_digit(8) {
            text.push(self.advance());
            if self.cursor.is_at_end() {
                break;
            }
        }

        let span_end = self.column;

        Token {
            tk_type: TokenKind::Number,
            value: text,
            address: Address::span(self.line, span_start..span_end, self.file_path.clone()),
        }
    }

    /// Scans binary numbers `0b{pattern}`
    fn scan_binary_number(&mut self) -> Token {
        // Start of span
        let span_start = self.column;
        // Skip 'b'
        self.advance();
        // Number text
        let mut text: String = String::from("0b");

        while self.cursor.peek().is_digit(2) {
            text.push(self.advance());
            if self.cursor.is_at_end() {
                break;
            }
        }

        let span_end = self.column;

        Token {
            tk_type: TokenKind::Number,
            value: text,
            address: Address::span(self.line, span_start..span_end, self.file_path.clone()),
        }
    }

    /// Scans identifier, and checks if it is keyword.
    /// Returns token with kind Identifier or Keyword.
    ///
    /// # Arguments
    ///
    /// * `start`: starting char of token
    ///
    fn scan_id_or_keyword(&mut self, start: char) -> Token {
        // Start of span
        let span_start = self.column;
        // Id/keyword text
        let mut text: String = String::from(start);

        while self.is_id(self.cursor.peek()) {
            text.push(self.advance());
            if self.cursor.is_at_end() {
                break;
            }
        }

        let tk_type: TokenKind = self
            .keywords
            .get(text.as_str())
            .cloned()
            .unwrap_or(TokenKind::Id);

        let span_end = self.column;

        Token {
            tk_type,
            value: text,
            address: Address::span(self.line, span_start..span_end, self.file_path.clone()),
        }
    }

    /// Adds 1 to `line` and resets to zero `column`
    fn new_line(&mut self) {
        self.line += 1;
        self.column = 0;
    }

    /// Eats character from cursor and returns it,
    /// adding 1 to `column` and `cursor.current`
    fn advance(&mut self) -> char {
        let ch: char = self.cursor.char_at(0);
        self.cursor.current += 1;
        self.column += 1;
        ch
    }

    /// Checking current character is equal to `ch`
    /// If current character is equal to `ch` advances it
    #[allow(clippy::wrong_self_convention)]
    fn is_match(&mut self, ch: char) -> bool {
        if !self.cursor.is_at_end() && self.cursor.char_at(0) == ch {
            self.advance();
            return true;
        }
        false
    }

    /// Creates token from tk_type and tk_value, then adds it to the tokens list
    fn add_tk(&mut self, tk_type: TokenKind, tk_value: &str) {
        self.tokens.push(Token::new(
            tk_type,
            tk_value.to_string(),
            Address::new(self.line, self.column, self.file_path.clone()),
        ));
    }

    /// Checks character is '0..9'
    fn is_digit(&self, ch: char) -> bool {
        ch.is_ascii_digit()
    }

    /// Checks character is 'a..z', 'A..Z', '_'
    fn is_letter(&self, ch: char) -> bool {
        ch.is_ascii_lowercase() || ch.is_ascii_uppercase() || (ch == '_')
    }

    /// Returns true if character is id.
    ///
    /// Character is id, if:
    /// - char is letter
    /// - char is digit
    /// - char is colon and next char is id
    fn is_id(&self, ch: char) -> bool {
        self.is_letter(ch) || self.is_digit(ch) || (ch == ':' && self.is_id(self.cursor.next()))
    }
}
