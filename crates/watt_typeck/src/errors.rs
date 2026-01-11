/// Imports
use ecow::EcoString;
use miette::{Diagnostic, NamedSource, SourceSpan};
use std::sync::Arc;
use thiserror::Error;
use watt_ast::ast::{BinaryOp, UnaryOp};

/// Typechecking related
#[derive(Debug, Error, Diagnostic)]
pub(crate) enum TypeckRelated {
    #[error("here...")]
    #[diagnostic(severity(hint))]
    Here {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label()]
        span: SourceSpan,
    },
    #[error("this type is {t:?}")]
    #[diagnostic(severity(hint))]
    ThisType {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label()]
        span: SourceSpan,
        t: String,
    },
}

/// Typechecking error
#[derive(Debug, Error, Diagnostic)]
pub(crate) enum TypeckError {
    #[error("could not resolve `{name}`.")]
    #[diagnostic(
        code(typeck::could_not_resolve),
        help("check symbol/variable existence.")
    )]
    CouldNotResolve {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this is not defined in the current scope.")]
        span: SourceSpan,
        name: EcoString,
    },
    #[error("could not assign value to a constant.")]
    #[diagnostic(code(typeck::could_not_assign_constant))]
    CouldNotAssignConstant {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this is unavailable.")]
        span: SourceSpan,
    },
    #[error("could not use value `{v}` as a type.")]
    #[diagnostic(code(typeck::could_not_use_value_as_type))]
    CouldNotUseValueAsType {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("could not use as type.")]
        span: SourceSpan,
        v: EcoString,
    },
    #[error("variable `{name}` is already defined.")]
    #[diagnostic(
        code(typeck::variable_is_already_defined),
        help("you can't declare two variables with the same name.")
    )]
    VariableIsAlreadyDefined {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this variable is already defined.")]
        span: SourceSpan,
        name: EcoString,
    },
    #[error("invalid binary operation `{op:?}` on types `{a}` & `{b}`.")]
    #[diagnostic(code(typeck::invalid_binary_op))]
    InvalidBinaryOp {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this binary operation is incorrect.")]
        span: SourceSpan,
        a: String,
        b: String,
        op: BinaryOp,
    },
    #[error("could not use `as` operator with `{a:?}` & `{b:?}`.")]
    #[diagnostic(
        code(typeck::as_with_non_primitives),
        help("only primitive types can be used with as operator.")
    )]
    InvalidAsOp {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this `as` operation is incorrect.")]
        span: SourceSpan,
        a: String,
        b: String,
    },
    #[error("could not cast `{a}` to `{b}`.")]
    #[diagnostic(code(typeck::could_not_cast))]
    CouldNotCast {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this `as` operation is incorrect.")]
        span: SourceSpan,
        a: String,
        b: String,
    },
    #[error("invalid unary operation `{op:?}` on type `{t}`.")]
    #[diagnostic(code(typeck::invalid_unary_op))]
    InvalidUnaryOp {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this unary operation is incorrect.")]
        span: SourceSpan,
        t: String,
        op: UnaryOp,
    },
    #[error("field `{field}` is not defined in struct `{t}`.")]
    #[diagnostic(code(typeck::field_is_not_defined))]
    FieldIsNotDefined {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this access is invalid.")]
        span: SourceSpan,
        t: EcoString,
        field: EcoString,
    },
    #[error("field `{field}` is not defined in the enum `{t:?}`")]
    #[diagnostic(code(typeck::enum_variant_field_is_not_defined))]
    EnumVariantFieldIsNotDefined {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this pattern isn't valid.")]
        span: SourceSpan,
        t: String,
        field: EcoString,
    },
    #[error("variable `{field}` is not defined in the module `{m}`.")]
    #[diagnostic(code(typeck::module_field_is_not_defined))]
    ModuleFieldIsNotDefined {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this access is invalid.")]
        span: SourceSpan,
        m: EcoString,
        field: EcoString,
    },
    #[error("type `{def}` is private.")]
    #[diagnostic(code(typeck::type_is_private))]
    TypeIsPrivate {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("usage of this type is incorrect.")]
        span: SourceSpan,
        def: String,
    },
    #[error("module field `{name}` is private.")]
    #[diagnostic(code(typeck::module_field_is_private))]
    ModuleFieldIsPrivate {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this access is invalid.")]
        span: SourceSpan,
        name: EcoString,
    },
    #[error("could not call `{t}`.")]
    #[diagnostic(code(typeck::could_not_call))]
    CouldNotCall {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this call operation is incorrect.")]
        span: SourceSpan,
        t: String,
    },
    #[error("could not resolve fields in `{t}`.")]
    #[diagnostic(code(typeck::could_not_resolve_fileds_in))]
    CouldNotResolveFieldsIn {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this is incorrect.")]
        span: SourceSpan,
        t: String,
    },
    #[error("type named `{t}` is not defined.")]
    #[diagnostic(code(typeck::type_is_not_defined))]
    TypeIsNotDefined {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this type is not defined.")]
        span: SourceSpan,
        t: EcoString,
    },
    #[error("module `{m}` is not defined.")]
    #[diagnostic(
        code(typeck::module_is_not_defined),
        help("please, file an issue on github."),
        url("https://github.com/watt-rs/watt")
    )]
    ModuleIsNotDefined { m: EcoString },
    #[error("module `{m}` is unknown and can't be imported.")]
    #[diagnostic(code(typeck::import_of_unknown_module))]
    ImportOfUnknownModule {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this module is unknown.")]
        span: SourceSpan,
        m: EcoString,
    },
    #[error("type named `{t}` is already defined.")]
    #[diagnostic(code(typeck::type_is_already_defined))]
    TypeIsAlreadyDefined {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("redefinition occurs here.")]
        span: SourceSpan,
        t: EcoString,
    },
    #[error("module `{m}` is already imported as `{name}`.")]
    #[diagnostic(code(typeck::module_is_already_imported))]
    ModuleIsAlreadyImportedAs {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this module is already imported.")]
        span: SourceSpan,
        m: EcoString,
        name: EcoString,
    },
    #[error("name `{name}` is already imported as `{def}`.")]
    #[diagnostic(code(typeck::def_is_already_imported))]
    DefIsAlreadyImported {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this name is already imported.")]
        span: SourceSpan,
        name: EcoString,
        def: String,
    },
    #[error("expected a logical epxression in if.")]
    #[diagnostic(code(typeck::expected_logical_in_if))]
    ExpectedLogicalInIf {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("expected logical expression in if.")]
        span: SourceSpan,
    },
    #[error("types missmatch. expected `{expected}`, got `{got}`.")]
    #[diagnostic(code(typeck::types_missmatch))]
    TypesMissmatch {
        #[related]
        related: Vec<TypeckRelated>,
        expected: String,
        got: String,
    },
    #[error("wrong unwrap pattern. expected variant of enum, got `{got}`")]
    #[diagnostic(code(typeck::wrong_unwrap_pattern))]
    WrongUnwrapPattern {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this seems to be wrong.")]
        span: SourceSpan,
        got: String,
    },
    #[error("wrong variant pattern. expected variant of enum, got `{got}`")]
    #[diagnostic(code(typeck::wrong_variant_pattern))]
    WrongVariantPattern {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this seems to be wrong.")]
        span: SourceSpan,
        got: String,
    },
    #[error("unexpected resolution `{res}`.")]
    #[diagnostic(code(typeck::unexpected_resolution), help("can't use `{res}` here."))]
    UnexpectedResolution {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this is unexpected.")]
        span: SourceSpan,
        res: String,
    },
    #[error("unexpected expr in resolution `{expr}`.")]
    #[diagnostic(
        code(typeck::unexpected_expr_in_resolution),
        help("please, file an issue on github."),
        url("https://github.com/watt-rs/watt")
    )]
    UnexpectedExprInResolution { expr: EcoString },
    #[error("arity missmatch. expected {expected}, got {got}")]
    #[diagnostic(code(typeck::arity_missmatch))]
    ArityMissmatch {
        #[related]
        related: Vec<TypeckRelated>,
        expected: usize,
        got: usize,
    },
    #[error("found recursive type `{t}`.")]
    #[diagnostic(
        code(typeck::types_recursion),
        help("types recursion is not supported.")
    )]
    RecursiveType {
        #[related]
        related: Vec<TypeckRelated>,
        t: String,
    },
}

/// Exhaustiveness error
#[derive(Debug, Error, Diagnostic)]
pub enum ExError {
    #[error("enum patterns missmatch.")]
    #[diagnostic(code(ex::enum_patterns_missmatch))]
    EnumPatternsMissmatch {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("patterns are missmatched.")]
        span: SourceSpan,
    },
    #[error("enum fields missmatch.")]
    #[diagnostic(code(ex::enum_unwrap_fields_missmatch))]
    EnumUnwrapFieldsMissmatch {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("fields of patterns are missmatched.")]
        span: SourceSpan,
    },
}
