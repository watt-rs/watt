/// Imports
use miette::{Diagnostic, NamedSource, SourceSpan};
use oil_lex::tokens::TokenKind;
use thiserror::Error;

/// Parse errors with `thiserror`
#[derive(Debug, Error, Diagnostic)]
pub enum ParseError {
    #[error("unexpected token {unexpected}.")]
    #[diagnostic(code(parse::unexpected_token), help("expected {expected:?}."))]
    UnexpectedToken {
        #[source_code]
        src: NamedSource<String>,
        #[label("this token is unexpected.")]
        span: SourceSpan,
        unexpected: String,
        expected: TokenKind,
    },
    #[error("unexpected end of file.")]
    #[diagnostic(code(parse::unexpected_eof))]
    UnexpectedEof,
    #[error("unexpected {unexpected} as access.")]
    #[diagnostic(code(parse::expected_id_as_access), help("expected identifier."))]
    ExpectedIdAsAccess {
        #[source_code]
        src: NamedSource<String>,
        #[label("this token is unexpected.")]
        span: SourceSpan,
        unexpected: String,
    },
    #[error("invalid operation in expression.")]
    #[diagnostic(
        code(parse::invalid_operation_in_expr),
        help("this operation available only in statements.")
    )]
    InvalidOperationInExpr {
        #[source_code]
        src: NamedSource<String>,
        #[label("this in unacceptable in expressions.")]
        span: SourceSpan,
        unexpected: String,
    },
    #[error("invalid \"=\" operation.")]
    #[diagnostic(
        code(parse::invalid_assign_operation),
        help("this operation can be used only after identifier.")
    )]
    InvalidAssignOperation {
        #[source_code]
        src: NamedSource<String>,
        #[label("this in unacceptable with \"=\" operation.")]
        span: SourceSpan,
    },
    #[error("unexpected assignment operator {unexpected:?}.")]
    #[diagnostic(
        code(parse::unexpected_assign_operation),
        help("please, file an issue on github."),
        url("https://github.com/oillanguage/oil")
    )]
    UnexpectedAssignmentOperator { unexpected: TokenKind },
    #[error("invalid \"{op}\" operation.")]
    #[diagnostic(
        code(parse::invalid_assign_operation),
        help("this operation can be used, only after identifier.")
    )]
    InvalidCompoundOperation {
        #[source_code]
        src: NamedSource<String>,
        #[label("this in unacceptable with \"{op}\" operation.")]
        span: SourceSpan,
        op: &'static str,
    },
    #[error("unexpected \"{unexpected}\" as expression.")]
    #[diagnostic(code(parse::invalid_assign_operation))]
    UnexpectedExpressionToken {
        #[source_code]
        src: NamedSource<String>,
        #[label("this can not be represented as expression.")]
        span: SourceSpan,
        unexpected: String,
    },
    #[error("unexpected node in type body.")]
    #[diagnostic(code(parse::invalid_assign_operation))]
    UnexpectedNodeInTypeBody {
        #[source_code]
        src: NamedSource<String>,
        #[label("type defined here.")]
        type_span: SourceSpan,
        #[label("this is unexpected in type body.")]
        span: SourceSpan,
    },
    #[error("unexpected \"{unexpected}\" as statement.")]
    #[diagnostic(
        code(parse::invalid_assign_operation),
        help("this operation can be used, only after identifier.")
    )]
    UnexpectedStatementToken {
        #[source_code]
        src: NamedSource<String>,
        #[label("this can not be represented as statement.")]
        span: SourceSpan,
        unexpected: String,
    },
}
