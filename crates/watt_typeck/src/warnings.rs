/// Imports
use miette::{Diagnostic, NamedSource, SourceSpan};
use std::sync::Arc;
use thiserror::Error;

/// Typeck warning
#[derive(Debug, Error, Diagnostic)]
pub(crate) enum TypeckWarning {
    #[error("non exhaustive expression.")]
    #[diagnostic(
        code(typeck::warn::non_exhaustive),
        help("type was equated to unit."),
        severity(warning)
    )]
    NonExhaustive {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label()]
        span: SourceSpan,
    },
    #[error("found todo.")]
    #[diagnostic(
        code(typeck::warn::found_todo),
        help("todo existence is ok, but this code will cause a panic when executed."),
        severity(warning)
    )]
    FoundTodo {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("found todo.")]
        span: SourceSpan,
    },
}
