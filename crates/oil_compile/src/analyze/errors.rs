use ecow::EcoString;
/// Imports
use miette::{Diagnostic, NamedSource, SourceSpan};
use oil_ir::ir::{IrBinaryOp, IrUnaryOp};
use thiserror::Error;

use crate::analyze::analyze::Typ;

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
}
