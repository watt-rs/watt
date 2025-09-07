/// Imports
use crate::analyze::{
    res::Res,
    typ::{CustomType, Typ},
};
use ecow::EcoString;
use miette::{Diagnostic, NamedSource, SourceSpan};
use oil_ir::ir::{IrBinaryOp, IrUnaryOp};
use std::sync::Arc;
use thiserror::Error;

/// For errors
unsafe impl Send for Typ {}
unsafe impl Sync for Typ {}
unsafe impl Send for Res {}
unsafe impl Sync for Res {}
unsafe impl Send for CustomType {}
unsafe impl Sync for CustomType {}

/// Analyze error
#[derive(Debug, Error, Diagnostic)]
pub enum AnalyzeError {
    #[error("could not resolve {name}.")]
    #[diagnostic(
        code(analyze::could_not_resolve),
        help("check symbol/variable existence.")
    )]
    CouldNotResolve {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this is not defined.")]
        span: SourceSpan,
        name: EcoString,
    },
    #[error("could not use type {t} as value.")]
    #[diagnostic(code(analyze::could_not_use_type_as_value))]
    CouldNotUseTypeAsValue {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("could not use as value.")]
        span: SourceSpan,
        t: EcoString,
    },
    #[error("could not use value {v} as type.")]
    #[diagnostic(code(analyze::could_not_use_value_as_type))]
    CouldNotUseValueAsType {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("could not use as type.")]
        span: SourceSpan,
        v: EcoString,
    },
    #[error("variable is already defined.")]
    #[diagnostic(
        code(analyze::variable_is_already_defined),
        help("you can not create two variables with same name.")
    )]
    VariableIsAlreadyDefined {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this variable is already defined.")]
        span: SourceSpan,
    },
    #[error("invalid binary operation {op:?} with types {a:?} & {b:?}.")]
    #[diagnostic(code(analyze::invalid_binary_op))]
    InvalidBinaryOp {
        #[source_code]
        src: NamedSource<Arc<String>>,
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
        src: NamedSource<Arc<String>>,
        #[label("this is incorrect.")]
        span: SourceSpan,
        t: Typ,
        op: IrUnaryOp,
    },
    #[error("could not access field of type {t:?}.")]
    #[diagnostic(code(analyze::invalid_field_access))]
    InvalidFieldAccess {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this is incorrect.")]
        span: SourceSpan,
        t: Typ,
    },
    #[error("field \"{field}\" is not defined in type {t}.")]
    #[diagnostic(code(analyze::field_is_not_defined))]
    FieldIsNotDefined {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this access is invalid.")]
        span: SourceSpan,
        t: EcoString,
        field: EcoString,
    },
    #[error("variant \"{variant}\" is not defined in enum \"{e}\".")]
    #[diagnostic(code(analyze::enum_variant_is_not_defined))]
    EnumVariantIsNotDefined {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this access is invalid.")]
        span: SourceSpan,
        e: EcoString,
        variant: EcoString,
    },
    #[error("field \"{field}\" is not defined in module \"{m}\".")]
    #[diagnostic(code(analyze::module_field_is_not_defined))]
    ModuleFieldIsNotDefined {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this access is invalid.")]
        span: SourceSpan,
        m: EcoString,
        field: EcoString,
    },
    #[error("field \"{field}\" is private in type \"{t}\".")]
    #[diagnostic(code(analyze::field_is_private))]
    FieldIsPrivate {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this access is invalid.")]
        span: SourceSpan,
        t: EcoString,
        field: EcoString,
    },
    #[error("type \"{t:?}\" is private.")]
    #[diagnostic(code(analyze::type_is_private))]
    TypeIsPrivate {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this access is invalid.")]
        span: SourceSpan,
        t: CustomType,
    },
    #[error("module field \"{name:?}\" is private.")]
    #[diagnostic(code(analyze::module_field_is_private))]
    ModuleFieldIsPrivate {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this access is invalid.")]
        span: SourceSpan,
        name: EcoString,
    },
    #[error("environments stack is empty. it`s a bug!")]
    #[diagnostic(
        code(analyze::environments_stack_is_empty),
        help("please, file an issue on github."),
        url("https://github.com/oillanguage/oil")
    )]
    EnvironmentsStackIsEmpty,
    #[error("could not call {res:?}.")]
    #[diagnostic(code(analyze::could_not_call))]
    CouldNotCall {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this is incorrect.")]
        span: SourceSpan,
        res: Res,
    },
    #[error("could not resolve fields in {res:?}.")]
    #[diagnostic(code(analyze::could_not_resolve_fileds_in))]
    CouldNotResolveFieldsIn {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this is incorrect.")]
        span: SourceSpan,
        res: Res,
    },
    #[error("type {t} is not defined.")]
    #[diagnostic(code(analyze::type_is_not_defined))]
    TypeIsNotDefined {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this type is not defined.")]
        span: SourceSpan,
        t: EcoString,
    },
    #[error("module {m} is not defined.")]
    #[diagnostic(
        code(analyze::module_is_not_defined),
        help("please, file an issue on github."),
        url("https://github.com/oillanguage/oil")
    )]
    ModuleIsNotDefined { m: EcoString },
    #[error("type named {t} is already defined.")]
    #[diagnostic(code(analyze::type_is_already_defined))]
    TypeIsAlreadyDefined {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("new definition here.")]
        span: SourceSpan,
        t: EcoString,
    },
    #[error("method named {m} is already defined.")]
    #[diagnostic(code(analyze::method_is_already_defined))]
    MethodIsAlreadyDefined {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this method is already defined.")]
        span: SourceSpan,
        m: EcoString,
    },
    #[error("invalid arguments.")]
    #[diagnostic(code(analyze::invalid_args))]
    InvalidArgs {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("parameters described here.")]
        params_span: SourceSpan,
        #[label("invalid arguments.")]
        span: SourceSpan,
    },
    #[error("expected a logical epxression in if.")]
    #[diagnostic(code(analyze::expected_logical_in_if))]
    ExpectedLogicalInIf {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("expected logical expression in if.")]
        span: SourceSpan,
    },
    #[error("expected a logical epxression in while.")]
    #[diagnostic(code(analyze::expected_logical_in_while))]
    ExpectedLogicalInWhile {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("expected logical expression in while.")]
        span: SourceSpan,
    },
    #[error("missmatched type annotation. expected {expected:?}, got {got:?}.")]
    #[diagnostic(code(analyze::missmatched_type_annotation))]
    MissmatchedTypeAnnotation {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("type annotation missmatched here.")]
        span: SourceSpan,
        expected: Typ,
        got: Typ,
    },
    #[error("types missmatch. expected {expected:?}, got {got:?}.")]
    #[diagnostic(code(analyze::types_missmatch))]
    TypesMissmatch {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("type annotation missmatched here.")]
        span: SourceSpan,
        expected: Typ,
        got: Typ,
    },
    #[error("call expression return type is void.")]
    #[diagnostic(code(analyze::call_expr_return_type_is_void))]
    CallExprReturnTypeIsVoid {
        #[source_code]
        fn_src: NamedSource<Arc<String>>,
        #[label("function defined here.")]
        definition_span: SourceSpan,
        #[source_code]
        call_src: NamedSource<Arc<String>>,
        #[label("function call occured here.")]
        span: SourceSpan,
    },
    #[error("break used outside loop.")]
    #[diagnostic(code(analyze::break_without_loop))]
    BreakWithoutLoop {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("not available here.")]
        span: SourceSpan,
    },
    #[error("continue used outside loop.")]
    #[diagnostic(code(analyze::continue_without_loop))]
    ContinueWithoutLoop {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("not available here.")]
        span: SourceSpan,
    },
    #[error("return used outside function.")]
    #[diagnostic(code(analyze::return_without_function))]
    ReturnWithoutFunction {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("not available here.")]
        span: SourceSpan,
    },
    #[error("wrong return type. expected {expected:?}, got {got:?}")]
    #[diagnostic(code(analyze::wrong_return_type))]
    WrongReturnType {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this return seems to be wrong.")]
        span: SourceSpan,
        expected: Typ,
        got: Typ,
    },
    #[error("unexpected resolution {res:?}.")]
    #[diagnostic(code(analyze::unexpected_resolution), help("can't use {res:?} here."))]
    UnexpectedResolution {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this is unexpected.")]
        span: SourceSpan,
        res: Res,
    },
    #[error("unexpected expr in resolution {expr:?}.")]
    #[diagnostic(
        code(analyze::unexpected_expr_in_resolution),
        help("please, file an issue on github."),
        url("https://github.com/oillanguage/oil")
    )]
    UnexpectedExprInResolution { expr: EcoString },
}
