use std::sync::Arc;

/// Imports
use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

/// Cfa error
#[derive(Debug, Error, Diagnostic)]
pub enum CfaError {
    #[error("break used outside loop.")]
    #[diagnostic(code(cfa::break_without_loop))]
    BreakWithoutLoop {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("not available here.")]
        span: SourceSpan,
    },
    #[error("continue used outside loop.")]
    #[diagnostic(code(cfa::continue_without_loop))]
    ContinueWithoutLoop {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("not available here.")]
        span: SourceSpan,
    },
    #[error("many default cases in one match.")]
    #[diagnostic(code(cfa::many_default_cases))]
    ManyDefaultCases {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this match contains many default \"_\" cases")]
        span: SourceSpan,
    },
    #[error("no default case found in match.")]
    #[diagnostic(code(cfa::no_default_case_found))]
    NoDefaultCaseFound {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this match doesn't contain any default case.")]
        span: SourceSpan,
    },
    #[error("not all of branches returns value.")]
    #[diagnostic(code(cfa::not_all_branches_returns_value))]
    NotAllOfBranchesReturnsValue {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this branches seems to have missing return.")]
        span: SourceSpan,
    },
}
