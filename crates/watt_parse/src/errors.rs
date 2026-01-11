/// Imports
use ecow::EcoString;
use miette::{Diagnostic, NamedSource, SourceSpan};
use std::sync::Arc;
use thiserror::Error;
use watt_lex::tokens::TokenKind;

/// Parse errors with `thiserror`
#[derive(Debug, Error, Diagnostic)]
pub(crate) enum ParseError {
    #[error("unexpected token `{unexpected}`.")]
    #[diagnostic(code(parse::unexpected_token), help("expected `{expected:?}`."))]
    UnexpectedToken {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this token is unexpected here.")]
        span: SourceSpan,
        unexpected: EcoString,
        expected: TokenKind,
    },
    #[error("expected semicolon after non-closing statement.")]
    #[diagnostic(
        code(parse::expected_semicolon),
        help("the semicolon can be omitted only after last statement in the block.")
    )]
    ExpectedSemicolon {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("expected semicolon after that.")]
        span: SourceSpan,
    },
    #[error("unexpected end of file.")]
    #[diagnostic(
        code(parse::unexpected_eof),
        help("please, file an issue on github."),
        url("https://github.com/watt-rs/watt")
    )]
    UnexpectedEof,
    #[error("could not represent `{op}` as assignment or compound operator.")]
    #[diagnostic(code(parse::invalid_assignment_operator))]
    InvalidAssignmentOperator {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this is unrecognized operator.")]
        span: SourceSpan,
        op: EcoString,
    },
    #[error("failed to parse assignment.")]
    #[diagnostic(code(parse::invalid_assignment_operation))]
    InvalidAssignmentOperation {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this in unacceptable with assignment operation.")]
        span: SourceSpan,
    },
    #[error("unexpected `{unexpected}` in expression parsing.")]
    #[diagnostic(code(parse::unexpected_expression_token))]
    UnexpectedExpressionToken {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this can not be represented as expression.")]
        span: SourceSpan,
        unexpected: EcoString,
    },
    #[error("unexpected `{unexpected}` in expression parsing.")]
    #[diagnostic(
        code(parse::unexpected_declaration_token),
        help("only `type`, `fn`, `extern`, `const` are declarations.")
    )]
    UnexpectedDeclarationToken {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this can not be represented as declaration.")]
        span: SourceSpan,
        unexpected: EcoString,
    },
    #[error("non-const value.")]
    #[diagnostic(
        code(parse::nonconst_expr),
        help(
            "constant values can't depend on the logical clauses,
            variables, functions, custom enums and types."
        )
    )]
    NonConstExpr {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this can not be used as a constant value.")]
        span: SourceSpan,
    },
}
