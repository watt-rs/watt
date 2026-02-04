/// Imports
use crate::cursor::Cursor;
use crate::errors::LexError;
use crate::tokens::*;
use ecow::EcoString;
use miette::NamedSource;
use std::collections::HashMap;
use std::sync::Arc;
use watt_common::address::Address;
use watt_common::{bail, skip};

/// Lexer structure
pub struct Lexer<'source, 'cursor> {
    /// Cursor, used to jump back and forth
    cursor: Cursor<'cursor>,
    /// Source code
    source: &'source Arc<NamedSource<String>>,
    /// Result tokens
    tokens: Vec<Token>,
    /// Keywords map
    keywords: HashMap<&'static str, TokenKind>,
}

/// Lexer implementation
impl<'source, 'cursor> Lexer<'source, 'cursor> {
    /// Creates new lexer from
    ///
    /// * `code`: source code represented as `&'cursor [char]`
    /// * `file_path`: source file path
    ///
    pub fn new(code: &'cursor [char], source: &'source Arc<NamedSource<String>>) -> Self {
        // Keywords list
        let keywords_map = HashMap::from([
            ("fn", TokenKind::Fn),
            ("if", TokenKind::If),
            ("elif", TokenKind::Elif),
            ("else", TokenKind::Else),
            ("type", TokenKind::Type),
            ("enum", TokenKind::Enum),
            ("loop", TokenKind::Loop),
            ("in", TokenKind::In),
            ("true", TokenKind::Bool),
            ("false", TokenKind::Bool),
            ("as", TokenKind::As),
            ("let", TokenKind::Let),
            ("use", TokenKind::Use),
            ("pub", TokenKind::Pub),
            ("match", TokenKind::Match),
            ("extern", TokenKind::Extern),
            ("for", TokenKind::For),
            ("panic", TokenKind::Panic),
            ("todo", TokenKind::Todo),
            ("const", TokenKind::Const),
        ]);
        // Lexer
        Lexer {
            cursor: Cursor::new(code),
            source,
            tokens: vec![],
            keywords: keywords_map,
        }
    }

    /// Converts source code represented as `&'cursor [char]`
    /// To a `Vec<Token>` - tokens list.
    pub fn lex(mut self) -> Vec<Token> {
        if !self.tokens.is_empty() {
            bail!(LexError::TokensListsNotEmpty);
        }
        while !self.cursor.is_at_end() {
            let ch = self.advance();
            match ch {
                '+' => {
                    if self.is_match('=') {
                        self.add_tk(TokenKind::AddAssign, "+=");
                    } else {
                        self.add_tk(TokenKind::Plus, "+");
                    }
                }
                '&' => {
                    if self.is_match('=') {
                        self.add_tk(TokenKind::AndEq, "&=");
                    } else if self.is_match('&') {
                        self.add_tk(TokenKind::And, "&&");
                    } else {
                        self.add_tk(TokenKind::Ampersand, "&");
                    }
                }
                '|' => {
                    if self.is_match('=') {
                        self.add_tk(TokenKind::OrEq, "|=");
                    } else if self.is_match('|') {
                        self.add_tk(TokenKind::Or, "||");
                    } else {
                        self.add_tk(TokenKind::Bar, "|");
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
                        self.add_tk(TokenKind::SubAssign, "-=");
                    } else if self.is_match('>') {
                        self.add_tk(TokenKind::Arrow, "->");
                    } else {
                        self.add_tk(TokenKind::Minus, "-");
                    }
                }
                '*' => {
                    if self.is_match('=') {
                        self.add_tk(TokenKind::MulAssign, "*=");
                    } else {
                        self.add_tk(TokenKind::Star, "*");
                    }
                }
                '%' => self.add_tk(TokenKind::Percent, "%"),
                '/' => {
                    // compound operator
                    if self.is_match('=') {
                        self.add_tk(TokenKind::DivAssign, "/=");
                    }
                    // line comment
                    else if self.is_match('/') {
                        while !self.is_match('\n') && !self.cursor.is_at_end() {
                            self.advance();
                        }
                    }
                    // multi-line comment
                    else if self.is_match('*') {
                        while !(self.cursor.peek() == '*'
                            && self.cursor.next() == '/'
                            && self.cursor.is_at_end())
                        {
                            if self.is_match('\n') {
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
                '(' => self.add_tk(TokenKind::Lparen, "("),
                ')' => self.add_tk(TokenKind::Rparen, ")"),
                '{' => self.add_tk(TokenKind::Lbrace, "{"),
                '}' => self.add_tk(TokenKind::Rbrace, "}"),
                '[' => self.add_tk(TokenKind::Lbracket, "["),
                ']' => self.add_tk(TokenKind::Rbracket, "]"),
                ',' => self.add_tk(TokenKind::Comma, ","),
                '.' => {
                    if self.is_match('.') {
                        self.add_tk(TokenKind::Range, "..")
                    } else {
                        self.add_tk(TokenKind::Dot, ".");
                    }
                }
                ':' => self.add_tk(TokenKind::Colon, ":"),
                ';' => self.add_tk(TokenKind::Semicolon, ";"),
                '<' => {
                    if self.is_match('=') {
                        self.add_tk(TokenKind::LessEq, "<=");
                    } else if self.is_match('>') {
                        self.add_tk(TokenKind::Concat, "<>");
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
                '\r' | '\t' | '\0' | ' ' | '\n' => skip!(),
                '\"' => {
                    let tk = self.scan_string();
                    self.tokens.push(tk)
                }
                '`' => {
                    let tk = self.scan_multiline_string();
                    self.tokens.push(tk);
                }
                '_' => self.add_tk(TokenKind::Wildcard, "_"),
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
                    else if self.is_letter(ch) {
                        let token = self.scan_id_or_keyword(ch);
                        self.tokens.push(token);
                    }
                    // unexpected
                    else {
                        bail!(LexError::UnexpectedCharacter {
                            src: self.source.clone(),
                            span: self.cursor.current.into(),
                            ch
                        })
                    }
                }
            }
        }
        self.tokens
    }

    /// Scans unicode codepoint.
    fn scan_unicode_codepoint(&mut self, small: bool) -> char {
        // Escape sequence start, bumping back to the `u` or `U`
        let start_location = self.cursor.current - 1;
        let hex_digits_amount = if small { 4 } else { 8 };

        if !self.is_match('{') {
            bail!(LexError::InvalidEscapeSequence {
                src: self.source.clone(),
                span: (start_location..self.cursor.current).into(),
                cause: "expected unicode codepoint start `{`."
            })
        }
        let mut buffer = EcoString::new();
        for _ in 0..hex_digits_amount {
            let ch = self.advance();
            if !ch.is_ascii_hexdigit() {
                bail!(LexError::InvalidEscapeSequence {
                    src: self.source.clone(),
                    span: (start_location..self.cursor.current).into(),
                    cause: "expected hex digit."
                })
            }
            buffer.push(ch);
        }
        if !self.is_match('}') {
            bail!(LexError::InvalidEscapeSequence {
                src: self.source.clone(),
                span: (start_location..self.cursor.current).into(),
                cause: "expected unicode codepoint end `}`."
            })
        }
        let result = match char::from_u32(u32::from_str_radix(&buffer, 16).expect("Invalid hex")) {
            Some(c) => c,
            None => {
                bail!(LexError::InvalidEscapeSequence {
                    src: self.source.clone(),
                    span: (start_location..self.cursor.current).into(),
                    cause: "failed to convert `unciode char` into `u32`."
                })
            }
        };
        result
    }

    /// Scans byte codepoint.
    fn scan_byte_codepoint(&mut self) -> char {
        // Escape sequence start, bumping back to the `x`
        let start_location = self.cursor.current - 1;

        if !self.is_match('{') {
            bail!(LexError::InvalidEscapeSequence {
                src: self.source.clone(),
                span: (start_location..self.cursor.current).into(),
                cause: "expected byte codepoint start `{`."
            })
        }
        let mut buffer = EcoString::new();
        for _ in 0..2 {
            let ch = self.advance();
            if !ch.is_ascii_hexdigit() {
                bail!(LexError::InvalidEscapeSequence {
                    src: self.source.clone(),
                    span: (start_location..self.cursor.current).into(),
                    cause: "expected hex digit."
                })
            }
            buffer.push(ch);
        }
        if !self.is_match('}') {
            bail!(LexError::InvalidEscapeSequence {
                src: self.source.clone(),
                span: (start_location..self.cursor.current).into(),
                cause: "expected byte codepoint end `}`."
            })
        }
        let result = match char::from_u32(u32::from_str_radix(&buffer, 16).expect("Invalid hex")) {
            Some(c) => c,
            None => {
                bail!(LexError::InvalidEscapeSequence {
                    src: self.source.clone(),
                    span: (start_location..self.cursor.current).into(),
                    cause: "failed to convert `unciode char` into `u32`."
                })
            }
        };
        result
    }

    /// Scans escape sequence.
    fn scan_escape_sequence(&mut self) -> char {
        // Checking character kind.
        let ch = self.advance();

        match ch {
            'n' => '\n',
            'r' => '\r',
            '"' => '"',
            '`' => '`',
            '\\' => '\\',
            'u' => self.scan_unicode_codepoint(true),
            'U' => self.scan_unicode_codepoint(false),
            'x' => self.scan_byte_codepoint(),
            _ => bail!(LexError::UnknownEscapeSequence {
                src: self.source.clone(),
                span: (self.cursor.current - 1..self.cursor.current).into(),
            }),
        }
    }

    /// Scans string. Implies quote is already ate. Eats ending quote.
    fn scan_string(&mut self) -> Token {
        let start_location = self.cursor.current;
        let mut text: EcoString = EcoString::new();

        while self.cursor.peek() != '\"' {
            let ch = self.advance();

            // String escaping
            if ch == '\\' {
                text.push(self.scan_escape_sequence());
            } else {
                text.push(ch);
            }

            if self.cursor.is_at_end() || self.is_match('\n') {
                bail!(LexError::UnclosedStringQuotes {
                    src: self.source.clone(),
                    span: (start_location..self.cursor.current).into(),
                })
            }
        }

        self.advance();
        let end_location = self.cursor.current;

        Token {
            tk_type: TokenKind::Text,
            value: text,
            address: Address::span(self.source.clone(), start_location..end_location),
        }
    }

    /// Scans multiline string. Implies quote is already ate. Eats ending quote.
    fn scan_multiline_string(&mut self) -> Token {
        let start_location = self.cursor.current;
        let mut text: EcoString = EcoString::new();

        while self.cursor.peek() != '`' {
            let ch = self.advance();

            if ch == '\\' {
                text.push(self.scan_escape_sequence());
            } else {
                text.push(ch);
            }

            if self.cursor.is_at_end() {
                bail!(LexError::UnclosedStringQuotes {
                    src: self.source.clone(),
                    span: (start_location..self.cursor.current).into(),
                })
            }
        }

        self.advance();
        let end_location = self.cursor.current;

        Token {
            tk_type: TokenKind::Text,
            value: text,
            address: Address::span(self.source.clone(), start_location..end_location),
        }
    }

    /// Scans decimal and integer numbers
    ///
    /// # Arguments
    /// * `start`: starting char of token
    ///
    fn scan_number(&mut self, start: char) -> Token {
        let start_location = self.cursor.current - 1;
        let mut text: EcoString = EcoString::from(start);
        let mut is_float: bool = false;

        while self.is_digit(self.cursor.peek()) || self.cursor.peek() == '.' {
            if self.cursor.peek() == '.' {
                // Checking start of range
                if self.cursor.next() == '.' {
                    break;
                }
                text.push(self.advance());

                // If number is float already, so here's
                // second dot in number what's incorrect.
                if is_float {
                    bail!(LexError::InvalidNumber {
                        src: self.source.clone(),
                        span: (start_location..self.cursor.current + 1).into(),
                        number: text
                    })
                }
                is_float = true;

                continue;
            }
            text.push(self.advance());
            if self.cursor.is_at_end() {
                break;
            }
        }

        let end_location = self.cursor.current;

        Token {
            tk_type: TokenKind::Number,
            value: text,
            address: Address::span(self.source.clone(), start_location..end_location),
        }
    }

    /// Scans hexadecimal numbers `0x{pattern}`
    fn scan_hexadecimal_number(&mut self) -> Token {
        let start_location = self.cursor.current - 1;
        // Skip 'x'
        self.advance();
        let mut text: EcoString = EcoString::from("0x");

        while self.cursor.peek().is_ascii_hexdigit() {
            text.push(self.advance());
            if self.cursor.is_at_end() {
                break;
            }
        }

        let end_location = self.cursor.current;

        Token {
            tk_type: TokenKind::Number,
            value: text,
            address: Address::span(self.source.clone(), start_location..end_location),
        }
    }

    /// Scans octal numbers `0o{pattern}`
    fn scan_octal_number(&mut self) -> Token {
        let start_location = self.cursor.current - 1;
        // Skip 'o'
        self.advance();
        let mut text: EcoString = EcoString::from("0o");

        while self.cursor.peek().is_digit(8) {
            text.push(self.advance());
            if self.cursor.is_at_end() {
                break;
            }
        }

        let end_location = self.cursor.current;

        Token {
            tk_type: TokenKind::Number,
            value: text,
            address: Address::span(self.source.clone(), start_location..end_location),
        }
    }

    /// Scans binary numbers `0b{pattern}`
    fn scan_binary_number(&mut self) -> Token {
        let start_location = self.cursor.current - 1;
        // Skip 'b'
        self.advance();
        let mut text: EcoString = EcoString::from("0b");

        while self.cursor.peek().is_digit(2) {
            text.push(self.advance());
            if self.cursor.is_at_end() {
                break;
            }
        }

        let end_location = self.cursor.current;

        Token {
            tk_type: TokenKind::Number,
            value: text,
            address: Address::span(self.source.clone(), start_location..end_location),
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
        let start_location = self.cursor.current - 1;
        let mut text: EcoString = EcoString::from(start);

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

        let end_location = self.cursor.current;

        Token {
            tk_type,
            value: text,
            address: Address::span(self.source.clone(), start_location..end_location),
        }
    }

    /// Eats character from cursor and returns it.
    /// Adds 1 to `column` and `cursor.current`
    fn advance(&mut self) -> char {
        self.cursor.bump()
    }

    /// Checks current character is equal to `ch`.
    /// If current character is equal to `ch` advances it
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
            tk_value.into(),
            Address::new(self.source.clone(), self.cursor.current),
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
    fn is_id(&self, ch: char) -> bool {
        self.is_letter(ch) || self.is_digit(ch)
    }
}
