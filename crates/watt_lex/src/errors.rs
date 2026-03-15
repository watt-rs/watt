/// Imports
use miette::{Diagnostic, NamedSource, SourceSpan};
use std::sync::Arc;
use thiserror::Error;

/// Defines lexer error
#[derive(Error, Diagnostic, Debug)]
pub enum LexError<'a> {
    /// Unexpected char
    #[error("unexpected character `{ch}`.")]
    #[diagnostic(code(lex::unexpected_char))]
    UnexpectedChar {
        ch: char,
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("try to remove this character.")]
        span: SourceSpan,
    },
    /// Unclosed string quotes
    #[error("found unclosed string quotes.")]
    #[diagnostic(code(lex::unclosed_string_quotes))]
    UnclosedStringQuotes {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("close string quotes by appending missed quote `\"`.")]
        span: SourceSpan,
    },
    /// Invalid float
    #[error("invalid float number.")]
    #[diagnostic(code(lex::invalid_float_number), severity(bug))]
    InvalidFloat {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this float number seems to be invalid.")]
        span: SourceSpan,
    },
    /// Invalid escape sequence
    #[error("invalid escape sequence.")]
    #[diagnostic(code(lex::invalid_escape_sequence), help("{cause}"))]
    InvalidEscapeSequence {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this escape sequence isn't valid.")]
        span: SourceSpan,
        cause: &'a str,
    },
    /// Unknown escape sequence
    #[error("unknown escape sequence.")]
    #[diagnostic(code(lex::unknown_escape_sequence))]
    UnknownEscapeSequence {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this escape sequence isn't valid.")]
        span: SourceSpan,
    },
}
