/// Imports
use miette::{Diagnostic, NamedSource, SourceSpan};
use oil_ast::ast::Node;
use thiserror::Error;

/// ir errors with `thiserror`
#[derive(Debug, Error, Diagnostic)]
pub enum IrError {
    #[error("unexpected \"{unexpected:?}\" as expression.")]
    #[diagnostic(
        code(uir::unexpected_expression_node),
        help("please, file an issue on github."),
        url("https://github.com/oillanguage/oil")
    )]
    UnexpectedExpressionNode { unexpected: Node },
    #[error("unexpected \"{unexpected:?}\" as statement.")]
    #[diagnostic(
        code(uir::unexpected_expression_node),
        help("please, file an issue on github."),
        url("https://github.com/oillanguage/oil")
    )]
    UnexpectedStatementNode { unexpected: Node },
    #[error("unexpected \"{unexpected:?}\" as block.")]
    #[diagnostic(
        code(uir::unexpected_block_node),
        help("please, file an issue on github."),
        url("https://github.com/oillanguage/oil")
    )]
    UnexpectedBlockNode { unexpected: Node },
    #[error("unexpected \"{unexpected:?}\" as declaration.")]
    #[diagnostic(
        code(uir::unexpected_block_node),
        help("please, file an issue on github."),
        url("https://github.com/oillanguage/oil")
    )]
    UnexpectedDeclarationNode { unexpected: Node },
    #[error("unexpected \"{unexpected:?}\" in type body.")]
    #[diagnostic(
        code(uir::unexpected_block_node),
        help("please, file an issue on github."),
        url("https://github.com/oillanguage/oil")
    )]
    UnexpectedNodeInTypebody { unexpected: Node },
    #[error("failed to parse f64.")]
    #[diagnostic(code(uir::failed_to_parse_f64))]
    FailedToParseF64 {
        #[source_code]
        src: NamedSource<String>,
        #[label("this number cannot be represented as f64.")]
        span: SourceSpan,
    },
    #[error("failed to parse i64.")]
    #[diagnostic(code(uir::failed_to_parse_i64))]
    FailedToParseI64 {
        #[source_code]
        src: NamedSource<String>,
        #[label("this number cannot be represented as i64.")]
        span: SourceSpan,
    },
    #[error("failed to parse operator.")]
    #[diagnostic(
        code(uir::unknown_op),
        help("please, file an issue on github."),
        url("https://github.com/oillanguage/oil")
    )]
    UnknownOp {
        #[source_code]
        src: NamedSource<String>,
        #[label("this operator is unknown.")]
        span: SourceSpan,
    },
}
