/// Modules
#[allow(unused_assignments)]
mod errors;
pub mod token;

/// Imports
use crate::{
    errors::LexError,
    token::{Span, Token, TokenKind},
};
use miette::NamedSource;
use std::{str::Chars, sync::Arc};
use watt_macros::bail;

/// Represents lexer
pub struct Lexer<'s> {
    /// Current file source
    /// used to report errors
    source: Arc<NamedSource<String>>,

    /// Source that we lexing
    src: Chars<'s>,

    /// Current and next chars
    idx: usize,
    current: Option<char>,
    next: Option<char>,
}

/// Implementation
impl<'s> Lexer<'s> {
    /// Creates new lexer
    pub fn new(file: Arc<NamedSource<String>>, source: &'s str) -> Self {
        let mut chars = source.chars();
        let (current, next) = (chars.next(), chars.next());
        Self {
            source: file,
            src: chars,
            current,
            next,
            idx: 0,
        }
    }

    fn advance(&mut self) {
        self.current = self.next.take();
        self.next = self.src.next();
        self.idx += 1;
    }

    /// Advances char and returns token
    fn advance_with(&mut self, tk: TokenKind, lexeme: &str) -> Token {
        self.advance();
        Token::new(
            Span(self.source.clone(), self.idx - 1..self.idx),
            tk,
            lexeme.to_string(),
        )
    }

    /// Advances char twice and returns token
    fn advance_twice_with(&mut self, tk: TokenKind, lexeme: &str) -> Token {
        self.advance();
        self.advance();
        Token::new(
            Span(self.source.clone(), self.idx - 2..self.idx),
            tk,
            lexeme.to_string(),
        )
    }

    /// Scans unicode codepoint.
    fn scan_unicode_codepoint(&mut self, small: bool) -> char {
        // Bumping `u`
        let start_span = self.idx;
        self.advance();

        // Calculating amount of hex digits
        let hex_digits_amount = if small { 4 } else { 8 };

        if self.current != Some('{') {
            bail!(LexError::InvalidEscapeSequence {
                src: self.source.clone(),
                span: (start_span..self.idx).into(),
                cause: "expected unicode codepoint start `{`."
            })
        }
        self.advance();

        let mut buffer = String::new();
        for _ in 0..hex_digits_amount {
            match self.current {
                Some(ch) => {
                    if !ch.is_ascii_hexdigit() {
                        bail!(LexError::InvalidEscapeSequence {
                            src: self.source.clone(),
                            span: (start_span..self.idx).into(),
                            cause: "expected hex digit."
                        })
                    }
                    self.advance();
                    buffer.push(ch);
                }
                None => bail!(LexError::InvalidEscapeSequence {
                    src: self.source.clone(),
                    span: (start_span..self.idx).into(),
                    cause: "unexpected EOF."
                }),
            }
        }

        if self.current != Some('}') {
            bail!(LexError::InvalidEscapeSequence {
                src: self.source.clone(),
                span: (start_span..self.idx).into(),
                cause: "expected unicode codepoint end `}`."
            })
        }
        self.advance();

        let result = match char::from_u32(u32::from_str_radix(&buffer, 16).expect("Invalid hex")) {
            Some(c) => c,
            None => {
                bail!(LexError::InvalidEscapeSequence {
                    src: self.source.clone(),
                    span: (start_span..self.idx).into(),
                    cause: "failed to convert `unicode char` into `u32`."
                })
            }
        };
        result
    }

    /// Scans byte codepoint.
    fn scan_byte_codepoint(&mut self) -> char {
        // Bumping `x`
        let start_span = self.idx;
        self.advance();

        if self.current != Some('{') {
            bail!(LexError::InvalidEscapeSequence {
                src: self.source.clone(),
                span: (start_span..self.idx).into(),
                cause: "expected byte codepoint start `{`."
            })
        }
        self.advance();

        let mut buffer = String::new();
        for _ in 0..2 {
            match self.current {
                Some(ch) => {
                    if !ch.is_ascii_hexdigit() {
                        bail!(LexError::InvalidEscapeSequence {
                            src: self.source.clone(),
                            span: (start_span..self.idx).into(),
                            cause: "expected hex digit."
                        })
                    }
                    self.advance();
                    buffer.push(ch);
                }
                None => bail!(LexError::InvalidEscapeSequence {
                    src: self.source.clone(),
                    span: (start_span..self.idx).into(),
                    cause: "unexpected EOF."
                }),
            }
        }

        if self.current != Some('}') {
            bail!(LexError::InvalidEscapeSequence {
                src: self.source.clone(),
                span: (start_span..self.idx).into(),
                cause: "expected byte codepoint end `}`."
            })
        }
        self.advance();

        let result = match char::from_u32(u32::from_str_radix(&buffer, 16).expect("Invalid hex")) {
            Some(c) => c,
            None => {
                bail!(LexError::InvalidEscapeSequence {
                    src: self.source.clone(),
                    span: (start_span..self.idx).into(),
                    cause: "failed to convert `unicode char` into `u32`."
                })
            }
        };
        result
    }

    /// Advances escape sequence.
    fn advance_escape_sequence(&mut self) -> char {
        // `\` char
        self.advance();

        // Reading next character.
        let ch = self.current;
        self.advance();

        // Checking character kind.
        match ch {
            Some('n') => '\n',
            Some('r') => '\r',
            Some('"') => '"',
            Some('`') => '`',
            Some('\\') => '\\',
            Some('u') => self.scan_unicode_codepoint(true),
            Some('U') => self.scan_unicode_codepoint(false),
            Some('x') => self.scan_byte_codepoint(),
            _ => bail!(LexError::UnknownEscapeSequence {
                src: self.source.clone(),
                span: (self.idx - 1..self.idx).into()
            }),
        }
    }

    /// Advances string
    fn advance_string(&mut self) -> Token {
        // Advancing `"`
        self.advance();
        let start = self.idx;

        // Text buffer
        let mut buffer = String::new();

        // Building string before reaching `"`
        while self.current != Some('"') {
            // Checking for next char
            match &self.current {
                Some('\\') => buffer.push(self.advance_escape_sequence()),
                Some(_) => {
                    buffer.push(self.current.unwrap());
                    self.advance();
                }
                None => bail!(LexError::UnclosedStringQuotes {
                    src: self.source.clone(),
                    span: (start..self.idx).into(),
                }),
            }
        }

        // Advancing `"`
        self.advance();
        let end = self.idx;
        Token::new(
            Span(self.source.clone(), start..end),
            TokenKind::String,
            buffer,
        )
    }

    /// Advances number
    fn advance_number(&mut self) -> Token {
        // If number is float
        let mut is_float = false;
        let start = self.idx;

        // Text buffer
        let mut buffer = String::new();

        // Building number before reaching
        // non-digit char.
        while self.is_digit() && !self.is_eof() {
            buffer.push(self.current.unwrap());
            self.advance();
            // Checking for float dot
            if self.current == Some('.') {
                // If next is digit
                if self.next.map(|it| it.is_ascii_digit()).unwrap_or(false) {
                    // If already float
                    if is_float {
                        bail!(LexError::InvalidFloat {
                            src: self.source.clone(),
                            span: (start..self.idx).into(),
                        })
                    } else {
                        buffer.push('.');
                        self.advance();
                        is_float = true;
                    }
                }
                // If next dot
                else if self.next == Some('.') {
                    break;
                }
            }
        }
        let end = self.idx;
        Token::new(
            Span(self.source.clone(), start..end),
            TokenKind::Number,
            buffer,
        )
    }

    /// Token kind for id
    fn token_kind_for_id(value: &str) -> TokenKind {
        match value {
            "extern" => TokenKind::Struct,
            "enum" => TokenKind::Enum,
            "type" => TokenKind::Type,
            "let" => TokenKind::Let,
            "import" => TokenKind::Import,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "true" => TokenKind::Bool,
            "false" => TokenKind::Bool,
            "pub" => TokenKind::Pub,
            "fn" => TokenKind::Fn,
            "match" => TokenKind::Match,
            "as" => TokenKind::As,
            "for" => TokenKind::For,
            _ => TokenKind::Id,
        }
    }

    /// Advances id or keyword
    fn advance_id_or_kw(&mut self) -> Token {
        let start = self.idx;
        // Text buffer
        let mut buffer = String::new();
        // Building id before reaching
        // char that is not letter, not digit,
        // and not underscore.
        while (self.is_id_letter() || self.is_digit()) && !self.is_eof() {
            buffer.push(self.current.unwrap());
            self.advance();
        }
        let end = self.idx;
        Token::new(
            Span(self.source.clone(), start..end),
            Self::token_kind_for_id(&buffer),
            buffer,
        )
    }

    /// Skips comment
    fn skip_comment(&mut self) {
        // //
        self.advance();
        self.advance();
        while self.current != Some('\n') {
            self.advance();
        }
    }

    /// Skips multiline comment
    fn skip_multiline_comment(&mut self) {
        // /*
        self.advance();
        self.advance();
        while !(self.current == Some('*') && self.next == Some('/')) {
            self.advance();
        }
        // */
        self.advance();
        self.advance();
    }

    /// Skips whitespaces and comments
    fn skip_trivia(&mut self) {
        loop {
            // Skipping whitespaces
            while self.is_whitespace() {
                self.advance();
            }

            // Skipping comments
            if self.current == Some('/') {
                // Matching next char
                match self.next {
                    Some('*') => self.skip_multiline_comment(),
                    Some('/') => self.skip_comment(),
                    _ => break,
                }
            }

            break;
        }
    }

    /// Is whitespace
    fn is_whitespace(&mut self) -> bool {
        matches!(
            self.current,
            Some(' ') | Some('\n') | Some('\t') | Some('\r')
        )
    }

    /// Is id letter
    fn is_id_letter(&mut self) -> bool {
        matches!(self.current, Some(it) if it.is_ascii_alphabetic() || it == '_')
    }

    /// Is digit
    fn is_digit(&mut self) -> bool {
        matches!(self.current, Some(it) if it.is_ascii_digit())
    }

    /// Is end of file
    fn is_eof(&mut self) -> bool {
        self.current.is_none()
    }
}

/// Iterator implementation
impl<'s> Iterator for Lexer<'s> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        // Skipping trivia chars
        self.skip_trivia();

        // Matching current and next
        match (self.current, self.next) {
            (Some('+'), Some('=')) => Some(self.advance_twice_with(TokenKind::PlusEq, "+=")),
            (Some('-'), Some('=')) => Some(self.advance_twice_with(TokenKind::MinusEq, "-=")),
            (Some('*'), Some('=')) => Some(self.advance_twice_with(TokenKind::StarEq, "*=")),
            (Some('/'), Some('=')) => Some(self.advance_twice_with(TokenKind::SlashEq, "/=")),
            (Some('%'), Some('=')) => Some(self.advance_twice_with(TokenKind::PercentEq, "%=")),
            (Some('&'), Some('=')) => Some(self.advance_twice_with(TokenKind::AmpersandEq, "&=")),
            (Some('|'), Some('=')) => Some(self.advance_twice_with(TokenKind::BarEq, "|=")),
            (Some('^'), Some('=')) => Some(self.advance_twice_with(TokenKind::CaretEq, "^=")),
            (Some('&'), Some('&')) => Some(self.advance_twice_with(TokenKind::DoubleAmp, "&&")),
            (Some('|'), Some('|')) => Some(self.advance_twice_with(TokenKind::DoubleBar, "||")),
            (Some('='), Some('=')) => Some(self.advance_twice_with(TokenKind::DoubleEq, "==")),
            (Some('!'), Some('=')) => Some(self.advance_twice_with(TokenKind::BangEq, "!=")),
            (Some('>'), Some('=')) => Some(self.advance_twice_with(TokenKind::Ge, ">=")),
            (Some('<'), Some('=')) => Some(self.advance_twice_with(TokenKind::Le, "<=")),
            (Some('_'), _) => Some(self.advance_with(TokenKind::Wildcard, "_")),
            (Some('&'), _) => Some(self.advance_with(TokenKind::Ampersand, "&")),
            (Some('|'), _) => Some(self.advance_with(TokenKind::Bar, "|")),
            (Some('^'), _) => Some(self.advance_with(TokenKind::Caret, "^")),
            (Some('%'), _) => Some(self.advance_with(TokenKind::Percent, "^")),
            (Some('+'), _) => Some(self.advance_with(TokenKind::Plus, "+")),
            (Some('-'), _) => Some(self.advance_with(TokenKind::Minus, "-")),
            (Some('*'), _) => Some(self.advance_with(TokenKind::Star, "*")),
            (Some('/'), _) => Some(self.advance_with(TokenKind::Slash, "/")),
            (Some('!'), _) => Some(self.advance_with(TokenKind::Bang, "!")),
            (Some('='), _) => Some(self.advance_with(TokenKind::Eq, "=")),
            (Some('>'), _) => Some(self.advance_with(TokenKind::Gt, ">")),
            (Some('<'), _) => Some(self.advance_with(TokenKind::Lt, "<")),
            (Some('.'), _) => Some(self.advance_with(TokenKind::Dot, ".")),
            (Some(','), _) => Some(self.advance_with(TokenKind::Comma, ",")),
            (Some('{'), _) => Some(self.advance_with(TokenKind::Lbrace, "{")),
            (Some('}'), _) => Some(self.advance_with(TokenKind::Rbrace, "}")),
            (Some('('), _) => Some(self.advance_with(TokenKind::Lparen, "(")),
            (Some(')'), _) => Some(self.advance_with(TokenKind::Rparen, ")")),
            (Some(':'), _) => Some(self.advance_with(TokenKind::Colon, ":")),
            (Some(';'), _) => Some(self.advance_with(TokenKind::Semi, ";")),
            (Some('"'), _) => Some(self.advance_string()),
            (Some(ch), _) => {
                if self.is_digit() {
                    Some(self.advance_number())
                } else if self.is_id_letter() {
                    Some(self.advance_id_or_kw())
                } else {
                    bail!(LexError::UnexpectedChar {
                        ch,
                        src: self.source.clone(),
                        span: (self.idx..self.idx).into(),
                    })
                }
            }
            (_, _) => None,
        }
    }
}
