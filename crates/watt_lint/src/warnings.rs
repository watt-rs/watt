/// Imports
use ecow::EcoString;
use miette::{Diagnostic, NamedSource, SourceSpan};
use std::sync::Arc;
use thiserror::Error;

/// Lint warning
#[derive(Debug, Error, Diagnostic)]
pub(crate) enum LintWarning {
    #[error("block is empty.")]
    #[diagnostic(
        code(lint::warn::block_is_empty),
        severity(warning),
        help("it's will be better to use `todo` or `todo as` here.")
    )]
    EmptyBlock {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this block is empty.")]
        span: SourceSpan,
    },
    #[error("type name should be in `PascalCase`")]
    #[diagnostic(code(lint::warn::wrong_type_name), severity(warning))]
    WrongTypeName {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("wrong type name here...")]
        span: SourceSpan,
    },
    #[error("variant name should be in `PascalCase`")]
    #[diagnostic(code(lint::warn::variant_type_name), severity(warning))]
    WrongVariantName {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("wrong variant name here...")]
        span: SourceSpan,
    },
    #[error("function name should be in `snake_case`")]
    #[diagnostic(code(lint::warn::wrong_function_name), severity(warning))]
    WrongFunctionName {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("wrong function name here...")]
        span: SourceSpan,
    },
    #[error("variable name should be in `snake_case`")]
    #[diagnostic(code(lint::warn::wrong_variable_name), severity(warning))]
    WrongVariableName {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("wrong variable name here...")]
        span: SourceSpan,
    },
    #[error("too many parameters in the `{name}`")]
    #[diagnostic(code(lint::warn::too_many_params), severity(warning))]
    TooManyParams {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("too many parameters.")]
        span: SourceSpan,
        name: EcoString,
    },
    #[error("too many parameters in the anonymous function.")]
    #[diagnostic(code(lint::warn::too_many_params_in_an_fn), severity(warning))]
    TooManyParamsInAnFn {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("too many parameters.")]
        span: SourceSpan,
    },
}
