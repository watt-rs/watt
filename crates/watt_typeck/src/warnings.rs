/// Imports
use miette::{Diagnostic, NamedSource, SourceSpan};
use std::sync::Arc;
use thiserror::Error;

/// Typeck warning
#[derive(Debug, Error, Diagnostic)]
pub enum TypeckWarning {
    #[error("found unsafe runtime field access.")]
    #[diagnostic(
        code(typeck::warn::access_of_dyn_field),
        help("it's better to cast `dyn` before accessing its fields."),
        severity(warning)
    )]
    AccessOfDynField {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this is unsafe.")]
        span: SourceSpan,
    },
    #[error("found unsafe runtime call.")]
    #[diagnostic(
        code(typeck::warn::call_of_dyn),
        help("it's better to cast `dyn` before calling it."),
        severity(warning)
    )]
    CallOfDyn {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this is unsafe.")]
        span: SourceSpan,
    },
    #[error("found unsafe cast.")]
    #[diagnostic(
        code(typeck::warn::unit_and_dyn_unification),
        help("it's better to get rid of unification `unit` and `dyn`."),
        severity(warning)
    )]
    UnitAndDynUnification {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("unify this...")]
        first_span: SourceSpan,
        #[label("and this is unsafe.")]
        second_span: SourceSpan,
    },
    #[error("found todo.")]
    #[diagnostic(
        code(typeck::warn::found_todo),
        help("todo existence is ok, but this code will cause a panic when executed."),
        severity(warning)
    )]
    FoundTodo {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("found todo.")]
        span: SourceSpan,
    },
}
