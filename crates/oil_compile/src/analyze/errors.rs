/// Imports
use crate::analyze::analyze::Typ;
use ecow::EcoString;
use miette::{Diagnostic, NamedSource, SourceSpan};
use oil_ir::ir::{IrBinaryOp, IrUnaryOp};
use thiserror::Error;

/// For errors
unsafe impl Send for Typ {}
unsafe impl Sync for Typ {}

/// Analyze error
#[derive(Debug, Error, Diagnostic)]
pub enum AnalyzeError {
    #[error("variable is not defined.")]
    #[diagnostic(
        code(analyze::variable_is_not_defined),
        help("check variable existence.")
    )]
    VariableIsNotDefined {
        #[source_code]
        src: NamedSource<String>,
        #[label("this variable is not defined.")]
        span: SourceSpan,
    },
    #[error("variable is already defined.")]
    #[diagnostic(
        code(analyze::variable_is_already_defined),
        help("you can not create two variables with same name.")
    )]
    VariableIsAlreadyDefined {
        #[source_code]
        src: NamedSource<String>,
        #[label("this variable is already defined.")]
        span: SourceSpan,
    },
    #[error("invalid binary operation {op:?} with types {a:?} & {b:?}.")]
    #[diagnostic(code(analyze::invalid_binary_op))]
    InvalidBinaryOp {
        #[source_code]
        src: NamedSource<String>,
        #[label("this is incorrect.")]
        span: SourceSpan,
        a: Typ,
        b: Typ,
        op: IrBinaryOp,
    },
    #[error("invalid unary operation {op:?} with type {t:?}.")]
    #[diagnostic(code(analyze::invalid_unary_op))]
    InvalidUnaryOp {
        #[source_code]
        src: NamedSource<String>,
        #[label("this is incorrect.")]
        span: SourceSpan,
        t: Typ,
        op: IrUnaryOp,
    },
    #[error("could not access field of type {t:?}.")]
    #[diagnostic(code(analyze::invalid_field_access))]
    InvalidFieldAccess {
        #[source_code]
        src: NamedSource<String>,
        #[label("this is incorrect.")]
        span: SourceSpan,
        t: Typ,
    },
    #[error("environments stack is empty. it`s a bug!")]
    #[diagnostic(
        code(analyze::environments_stack_is_empty),
        help("please, file an issue on github."),
        url("https://github.com/oillanguage/oil")
    )]
    EnvironmentsStackIsEmpty,
    #[error("could not call {t:?}.")]
    #[diagnostic(code(analyze::could_not_call))]
    CouldNotCall {
        #[source_code]
        src: NamedSource<String>,
        #[label("this is incorrect.")]
        span: SourceSpan,
        t: Typ,
    },
    #[error("type {t:?} is not found.")]
    #[diagnostic(code(analyze::type_is_not_found))]
    TypeIsNotDefined {
        #[source_code]
        src: NamedSource<String>,
        #[label("this type is not found.")]
        span: SourceSpan,
        t: EcoString,
    },
    #[error("invalid arguments.")]
    #[diagnostic(code(analyze::invalid_args))]
    InvalidArgs {
        #[source_code]
        src: NamedSource<String>,
        #[label("parameters described here.")]
        params_span: SourceSpan,
        #[label("invalid arguments.")]
        span: SourceSpan,
    },
    #[error("invalid arguments.")]
    #[diagnostic(code(analyze::invalid_args))]
    ModuleIsNot {
        #[source_code]
        src: NamedSource<String>,
        #[label("parameters described here.")]
        params_span: SourceSpan,
        #[label("invalid arguments.")]
        span: SourceSpan,
    },
}
