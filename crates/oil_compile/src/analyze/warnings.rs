/// Imports
use miette::{Diagnostic, NamedSource, SourceSpan};
use std::sync::Arc;
use thiserror::Error;

/// Analyze warning
#[derive(Debug, Error, Diagnostic)]
pub enum AnalyzeWarning {
    #[error("found unsafe code.")]
    #[diagnostic(
        code(analyze::warn::access_of_dyn_field),
        help("unsafe runtime field access."),
        severity(warning)
    )]
    AccessOfDynField {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this is unsafe.")]
        span: SourceSpan,
    },
    #[error("found unsafe code.")]
    #[diagnostic(
        code(analyze::warn::call_of_dyn),
        help("unsafe runtime call."),
        severity(warning)
    )]
    CallOfDyn {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this is unsafe.")]
        span: SourceSpan,
    },
}
