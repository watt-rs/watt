/// Imports
use ecow::EcoString;
use miette::{Diagnostic, NamedSource, SourceSpan};
use std::sync::Arc;
use thiserror::Error;

/// Lex errors with `thiserror`
#[derive(Debug, Error, Diagnostic)]
pub enum LexError<'a> {
    #[error("unexpected character \"{ch}\".")]
    #[diagnostic(code(lex::unexpected_char))]
    UnexpectedCharacter {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this character isn't expected here.")]
        span: SourceSpan,
        ch: char,
    },
    #[error("unclosed string quotes.")]
    #[diagnostic(code(lex::unclosed_string_quotes))]
    UnclosedStringQuotes {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("no ending quote specified.")]
        span: SourceSpan,
    },
    #[error("number `{number}` isn't valid.")]
    #[diagnostic(code(lex::invalid_number))]
    InvalidNumber {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this number isn't valid.")]
        span: SourceSpan,
        number: EcoString,
    },
    #[error("tokens len isn't empty.")]
    #[diagnostic(
        code(lex::tokens_list_is_not_empty),
        help("please, file an issue on github."),
        url("https://github.com/watt-rs/watt")
    )]
    TokensListsNotEmpty,
    #[error("not a file provided.")]
    #[diagnostic(
        code(lex::not_a_file_provided),
        help("please, file an issue on github."),
        url("https://github.com/watt-rs/watt")
    )]
    NotAFileProvided,
    #[error("invalid escape sequence.")]
    #[diagnostic(code(lex::invalid_escape_sequence), help("{cause}"))]
    InvalidEscapeSequence {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this escape sequence isn't valid.")]
        span: SourceSpan,
        cause: &'a str,
    },
    #[error("unknown escape sequence.")]
    #[diagnostic(code(lex::unknown_escape_sequence))]
    UnknownEscapeSequence {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this escape sequence isn't valid.")]
        span: SourceSpan,
    },
}
