/// Imports
use crate::typ::{
    def::{ModuleDef, TypeDef},
    res::Res,
    typ::Typ,
};
use ecow::EcoString;
use miette::{Diagnostic, NamedSource, SourceSpan};
use std::sync::Arc;
use thiserror::Error;
use watt_ast::ast::{BinaryOp, UnaryOp};

/// For errors
/// todo: reimplement using derive.
unsafe impl Send for Typ {}
unsafe impl Sync for Typ {}
unsafe impl Send for Res {}
unsafe impl Sync for Res {}
unsafe impl Send for ModuleDef {}
unsafe impl Sync for ModuleDef {}
unsafe impl Send for TypeDef {}
unsafe impl Sync for TypeDef {}

/// Typechecking related
#[derive(Debug, Error, Diagnostic)]
pub enum TypeckRelated {
    #[error("this...")]
    #[diagnostic(severity(hint))]
    This {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label()]
        span: SourceSpan,
    },
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
        t: Typ,
    },
    #[error("with this.")]
    #[diagnostic(severity(hint))]
    WithThis {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label()]
        span: SourceSpan,
    },
}

/// Typechecking error
#[derive(Debug, Error, Diagnostic)]
pub enum TypeckError {
    #[error("could not resolve {name}.")]
    #[diagnostic(
        code(typeck::could_not_resolve),
        help("check symbol/variable existence.")
    )]
    CouldNotResolve {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this is not defined.")]
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
    #[error("could not unify {t1:?} and {t2:?}.")]
    #[diagnostic(code(typeck::could_not_unify))]
    CouldNotUnify {
        #[related]
        related: Vec<TypeckRelated>,
        t1: Typ,
        t2: Typ,
    },
    #[error("could not use value {v} as type.")]
    #[diagnostic(code(typeck::could_not_use_value_as_type))]
    CouldNotUseValueAsType {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("could not use as type.")]
        span: SourceSpan,
        v: EcoString,
    },
    #[error("variable is already defined.")]
    #[diagnostic(
        code(typeck::variable_is_already_defined),
        help("you can not create two variables with same name.")
    )]
    VariableIsAlreadyDefined {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this variable is already defined.")]
        span: SourceSpan,
    },
    #[error("invalid binary operation {op:?} with types {a:?} & {b:?}.")]
    #[diagnostic(code(typeck::invalid_binary_op))]
    InvalidBinaryOp {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this is incorrect.")]
        span: SourceSpan,
        a: Typ,
        b: Typ,
        op: BinaryOp,
    },
    #[error("could not use `as` with types {a:?} & {b:?}.")]
    #[diagnostic(
        code(typeck::as_with_non_primitive),
        help("only primitive types can be used with as operator.")
    )]
    InvalidAsOp {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this is incorrect.")]
        span: SourceSpan,
        a: Typ,
        b: Typ,
    },
    #[error("could not cast {a:?} to {b:?}.")]
    #[diagnostic(code(typeck::could_not_cast))]
    CouldNotCast {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this is incorrect.")]
        span: SourceSpan,
        a: Typ,
        b: Typ,
    },
    #[error("invalid unary operation {op:?} with type {t:?}.")]
    #[diagnostic(code(typeck::invalid_unary_op))]
    InvalidUnaryOp {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this is incorrect.")]
        span: SourceSpan,
        t: Typ,
        op: UnaryOp,
    },
    #[error("field \"{field}\" is not defined in type \"{t}\".")]
    #[diagnostic(code(typeck::field_is_not_defined))]
    FieldIsNotDefined {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this access is invalid.")]
        span: SourceSpan,
        t: EcoString,
        field: EcoString,
    },
    #[error("field \"{field}\" is not defined in {res:?}")]
    #[diagnostic(code(typeck::enum_variant_field_is_not_defined))]
    EnumVariantFieldIsNotDefined {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this pattern isn't valid.")]
        span: SourceSpan,
        res: Res,
        field: EcoString,
    },
    #[error("field \"{field}\" is not defined in module \"{m}\".")]
    #[diagnostic(code(typeck::module_field_is_not_defined))]
    ModuleFieldIsNotDefined {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this access is invalid.")]
        span: SourceSpan,
        m: EcoString,
        field: EcoString,
    },
    #[error("type \"{def:?}\" is private.")]
    #[diagnostic(code(typeck::type_is_private))]
    TypeIsPrivate {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this access is invalid.")]
        span: SourceSpan,
        def: TypeDef,
    },
    #[error("module field \"{name}\" is private.")]
    #[diagnostic(code(typeck::module_field_is_private))]
    ModuleFieldIsPrivate {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this access is invalid.")]
        span: SourceSpan,
        name: EcoString,
    },
    #[error("could not call {res:?}.")]
    #[diagnostic(code(typeck::could_not_call))]
    CouldNotCall {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this is incorrect.")]
        span: SourceSpan,
        res: Res,
    },
    #[error("could not resolve fields in {res:?}.")]
    #[diagnostic(code(typeck::could_not_resolve_fileds_in))]
    CouldNotResolveFieldsIn {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this is incorrect.")]
        span: SourceSpan,
        res: Res,
    },
    #[error("type \"{t}\" is not defined.")]
    #[diagnostic(code(typeck::type_is_not_defined))]
    TypeIsNotDefined {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this type is not defined.")]
        span: SourceSpan,
        t: EcoString,
    },
    #[error("module \"{m}\" is not defined.")]
    #[diagnostic(
        code(typeck::module_is_not_defined),
        help("please, file an issue on github."),
        url("https://github.com/wattlanguage/watt")
    )]
    ModuleIsNotDefined { m: EcoString },
    #[error("module \"{m}\" is unknown and can't be imported.")]
    #[diagnostic(code(typeck::import_of_unknown_module))]
    ImportOfUnknownModule {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this module is not unknown.")]
        span: SourceSpan,
        m: EcoString,
    },
    #[error("type named \"{t}\" is already defined.")]
    #[diagnostic(code(typeck::type_is_already_defined))]
    TypeIsAlreadyDefined {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("new definition here.")]
        span: SourceSpan,
        t: EcoString,
    },
    #[error("module \"{m}\" is already imported as \"{name}\".")]
    #[diagnostic(code(typeck::module_is_already_imported))]
    ModuleIsAlreadyImportedAs {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this module is already imported.")]
        span: SourceSpan,
        m: EcoString,
        name: EcoString,
    },
    #[error("name \"{name}\" is already imported as {def:?}.")]
    #[diagnostic(code(typeck::def_is_already_imported))]
    DefIsAlreadyImported {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this name is already imported.")]
        span: SourceSpan,
        name: EcoString,
        def: ModuleDef,
    },
    #[error("expected a logical epxression in if.")]
    #[diagnostic(code(typeck::expected_logical_in_if))]
    ExpectedLogicalInIf {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("expected logical expression in if.")]
        span: SourceSpan,
    },
    #[error("types missmatch. expected {expected:?}, got {got:?}.")]
    #[diagnostic(code(typeck::types_missmatch))]
    TypesMissmatch {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("types missmatched here.")]
        span: SourceSpan,
        expected: Typ,
        got: Typ,
    },
    #[error("wrong unwrap pattern. expected variant of enum, got {got:?}")]
    #[diagnostic(code(typeck::wrong_unwrap_pattern))]
    WrongUnwrapPattern {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this seems to be wrong.")]
        span: SourceSpan,
        got: Res,
    },
    #[error("wrong variant pattern. expected variant of enum, got {got:?}")]
    #[diagnostic(code(typeck::wrong_variant_pattern))]
    WrongVariantPattern {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this seems to be wrong.")]
        span: SourceSpan,
        got: Res,
    },
    #[error("unexpected resolution {res:?}.")]
    #[diagnostic(code(typeck::unexpected_resolution), help("can't use {res:?} here."))]
    UnexpectedResolution {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this is unexpected.")]
        span: SourceSpan,
        res: Res,
    },
    #[error("unexpected expr in resolution {expr:?}.")]
    #[diagnostic(
        code(typeck::unexpected_expr_in_resolution),
        help("please, file an issue on github."),
        url("https://github.com/wattlanguage/watt")
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
    #[error("found types recursion.")]
    #[diagnostic(code(typeck::types_recursion))]
    TypesRecursion {
        #[related]
        related: Vec<TypeckRelated>,
        t1: Typ,
        t2: Typ,
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
