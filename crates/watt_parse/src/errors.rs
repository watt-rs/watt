/// Imports
use ecow::EcoString;
use miette::{Diagnostic, NamedSource, SourceSpan};
use std::sync::Arc;
use thiserror::Error;
use watt_lex::tokens::TokenKind;

/// Parse errors with `thiserror`
#[derive(Debug, Error, Diagnostic)]
pub enum ParseError {
    #[error("unexpected token {unexpected}.")]
    #[diagnostic(code(parse::unexpected_token), help("expected {expected:?}."))]
    UnexpectedToken {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this token is unexpected.")]
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
    #[diagnostic(code(parse::unexpected_eof))]
    UnexpectedEof,
    #[error("unexpected {unexpected} as access.")]
    #[diagnostic(code(parse::expected_id_as_access), help("expected identifier."))]
    ExpectedIdAsAccess {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this token is unexpected.")]
        span: SourceSpan,
        unexpected: EcoString,
    },
    #[error("invalid operation in expression.")]
    #[diagnostic(
        code(parse::invalid_operation_in_expr),
        help("this operation available only in statements.")
    )]
    InvalidOperationInExpr {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this in unacceptable in expressions.")]
        span: SourceSpan,
        unexpected: EcoString,
    },
    #[error("could not represent \"{op}\" as assignment or compound operator.")]
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
    #[error("unexpected \"{unexpected}\" as expression.")]
    #[diagnostic(code(parse::unexpected_expression_token))]
    UnexpectedExpressionToken {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this can not be represented as expression.")]
        span: SourceSpan,
        unexpected: EcoString,
    },
    #[error("variable access can not be a statement.")]
    #[diagnostic(code(parse::variable_access_cannot_be_statement))]
    VariableAccessCanNotBeStatement {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this can not be a statement")]
        span: SourceSpan,
    },
    #[error("unexpected statement node.")]
    #[diagnostic(code(parse::unexpected_statement_node))]
    UnexpectedStatement {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this can not be a statement")]
        span: SourceSpan,
    },
    #[error("unexpected node in type body.")]
    #[diagnostic(code(parse::unexpected_node_in_type_body))]
    UnexpectedNodeInTypeBody {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("type defined here.")]
        type_span: SourceSpan,
        #[label("this is unexpected in type body.")]
        span: SourceSpan,
    },
    #[error("unexpected \"{unexpected}\" as statement.")]
    #[diagnostic(code(parse::unexpected_statement_token))]
    UnexpectedStatementToken {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this can not be represented as statement.")]
        span: SourceSpan,
        unexpected: EcoString,
    },
    #[error("unexpected \"{unexpected}\" as declaration.")]
    #[diagnostic(
        code(parse::unexpected_declaration_token),
        help("only \"type\", \"let\", and \"fn\" is declarations.")
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
        help("constant values can't depend on the logical clauses or variables.")
    )]
    NonConstExpr {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this can not be used in constant value.")]
        span: SourceSpan,
    },
}
