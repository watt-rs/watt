/// Imports
use crate::{
    resolve::{res::Res, resolve::ModDef},
    typ::{CustomType, Typ},
};
use ecow::EcoString;
use miette::{Diagnostic, NamedSource, SourceSpan};
use std::sync::Arc;
use thiserror::Error;
use watt_ast::ast::{BinaryOp, UnaryOp};

/// For errors
unsafe impl Send for Typ {}
unsafe impl Sync for Typ {}
unsafe impl Send for Res {}
unsafe impl Sync for Res {}
unsafe impl Send for CustomType {}
unsafe impl Sync for CustomType {}

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
        src: NamedSource<Arc<String>>,
        #[label("this is not defined.")]
        span: SourceSpan,
        name: EcoString,
    },
    #[error("could not unify {t1:?} and {t2:?}.")]
    #[diagnostic(code(typeck::could_not_unify))]
    CouldNotUnify {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("could not unify this...")]
        first_span: SourceSpan,
        t1: Typ,
        #[label("with this")]
        second_span: SourceSpan,
        t2: Typ,
    },
    #[error("could not unify trait {tr:?} and {ty:?}.")]
    #[diagnostic(
        code(typeck::could_not_unify_trait_and_typ),
        help(
            "all trait functions are `public` by default,
check that {ty:?} you trying to cast to the trait {tr:?}
implements all trait functions with `pub` modifier."
        )
    )]
    CouldNotUnifyTraitAndTyp {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("could not unify this...")]
        first_span: SourceSpan,
        tr: Typ,
        #[label("with this")]
        second_span: SourceSpan,
        ty: Typ,
    },
    #[error("could not use value {v} as type.")]
    #[diagnostic(code(typeck::could_not_use_value_as_type))]
    CouldNotUseValueAsType {
        #[source_code]
        src: NamedSource<Arc<String>>,
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
        src: NamedSource<Arc<String>>,
        #[label("this variable is already defined.")]
        span: SourceSpan,
    },
    #[error("invalid binary operation {op:?} with types {a:?} & {b:?}.")]
    #[diagnostic(code(typeck::invalid_binary_op))]
    InvalidBinaryOp {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this is incorrect.")]
        span: SourceSpan,
        a: Typ,
        b: Typ,
        op: BinaryOp,
    },
    #[error("invalid unary operation {op:?} with type {t:?}.")]
    #[diagnostic(code(typeck::invalid_unary_op))]
    InvalidUnaryOp {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this is incorrect.")]
        span: SourceSpan,
        t: Typ,
        op: UnaryOp,
    },
    #[error("field \"{field}\" is not defined in type \"{t}\".")]
    #[diagnostic(code(typeck::field_is_not_defined))]
    FieldIsNotDefined {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this access is invalid.")]
        span: SourceSpan,
        t: EcoString,
        field: EcoString,
    },
    #[error("variant \"{variant}\" is not defined in enum \"{e}\".")]
    #[diagnostic(code(typeck::enum_variant_is_not_defined))]
    EnumVariantIsNotDefined {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this access is invalid.")]
        span: SourceSpan,
        e: EcoString,
        variant: EcoString,
    },
    #[error("field \"{field}\" is not defined in {res:?}")]
    #[diagnostic(code(typeck::enum_variant_field_is_not_defined))]
    EnumVariantFieldIsNotDefined {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this pattern isn't valid.")]
        span: SourceSpan,
        res: Res,
        field: EcoString,
    },
    #[error("field \"{field}\" is not defined in module \"{m}\".")]
    #[diagnostic(code(typeck::module_field_is_not_defined))]
    ModuleFieldIsNotDefined {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this access is invalid.")]
        span: SourceSpan,
        m: EcoString,
        field: EcoString,
    },
    #[error("field \"{field}\" is private in type \"{t}\".")]
    #[diagnostic(code(typeck::field_is_private))]
    FieldIsPrivate {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this access is invalid.")]
        span: SourceSpan,
        t: EcoString,
        field: EcoString,
    },
    #[error("type \"{t:?}\" is private.")]
    #[diagnostic(code(typeck::type_is_private))]
    TypeIsPrivate {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this access is invalid.")]
        span: SourceSpan,
        t: CustomType,
    },
    #[error("module field \"{name}\" is private.")]
    #[diagnostic(code(typeck::module_field_is_private))]
    ModuleFieldIsPrivate {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this access is invalid.")]
        span: SourceSpan,
        name: EcoString,
    },
    #[error("environments stack is empty. it`s a bug!")]
    #[diagnostic(
        code(typeck::environments_stack_is_empty),
        help("please, file an issue on github."),
        url("https://github.com/wattlanguage/watt")
    )]
    EnvironmentsStackIsEmpty,
    #[error("could not call {res:?}.")]
    #[diagnostic(code(typeck::could_not_call))]
    CouldNotCall {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this is incorrect.")]
        span: SourceSpan,
        res: Res,
    },
    #[error("could not resolve fields in {res:?}.")]
    #[diagnostic(code(typeck::could_not_resolve_fileds_in))]
    CouldNotResolveFieldsIn {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this is incorrect.")]
        span: SourceSpan,
        res: Res,
    },
    #[error("type \"{t}\" is not defined.")]
    #[diagnostic(code(typeck::type_is_not_defined))]
    TypeIsNotDefined {
        #[source_code]
        src: NamedSource<Arc<String>>,
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
        src: NamedSource<Arc<String>>,
        #[label("this module is not unknown.")]
        span: SourceSpan,
        m: EcoString,
    },
    #[error("type named \"{t}\" is already defined.")]
    #[diagnostic(code(typeck::type_is_already_defined))]
    TypeIsAlreadyDefined {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("new definition here.")]
        span: SourceSpan,
        t: EcoString,
    },
    #[error("method named \"{m}\" is already defined.")]
    #[diagnostic(code(typeck::method_is_already_defined))]
    MethodIsAlreadyDefined {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this method is already defined.")]
        span: SourceSpan,
        m: EcoString,
    },
    #[error("module \"{m}\" is already imported as \"{name}\".")]
    #[diagnostic(code(typeck::module_is_already_imported))]
    ModuleIsAlreadyImportedAs {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this module is already imported.")]
        span: SourceSpan,
        m: EcoString,
        name: EcoString,
    },
    #[error("name \"{name}\" is already imported as {def:?}.")]
    #[diagnostic(code(typeck::def_is_already_imported))]
    DefIsAlreadyImported {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this name is already imported.")]
        span: SourceSpan,
        name: EcoString,
        def: ModDef,
    },
    #[error("expected a logical epxression in if.")]
    #[diagnostic(code(typeck::expected_logical_in_if))]
    ExpectedLogicalInIf {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("expected logical expression in if.")]
        span: SourceSpan,
    },
    #[error("expected a logical epxression in while.")]
    #[diagnostic(code(typeck::expected_logical_in_while))]
    ExpectedLogicalInWhile {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("expected logical expression in while.")]
        span: SourceSpan,
    },
    #[error("types missmatch. expected {expected:?}, got {got:?}.")]
    #[diagnostic(code(typeck::types_missmatch))]
    TypesMissmatch {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("types missmatched here.")]
        span: SourceSpan,
        expected: Typ,
        got: Typ,
    },
    #[error("call expression return type is void.")]
    #[diagnostic(code(typeck::call_expr_return_type_is_void))]
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
    #[error("wrong unwrap pattern. expected variant of enum, got {got:?}")]
    #[diagnostic(code(typeck::wrong_unwrap_pattern))]
    WrongUnwrapPattern {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this seems to be wrong.")]
        span: SourceSpan,
        got: Res,
    },
    #[error("wrong variant pattern. expected variant of enum, got {got:?}")]
    #[diagnostic(code(typeck::wrong_variant_pattern))]
    WrongVariantPattern {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this seems to be wrong.")]
        span: SourceSpan,
        got: Res,
    },
    #[error("unexpected resolution {res:?}.")]
    #[diagnostic(code(typeck::unexpected_resolution), help("can't use {res:?} here."))]
    UnexpectedResolution {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("this is unexpected.")]
        span: SourceSpan,
        res: Res,
    },
    #[error("could not use break outside loop.")]
    #[diagnostic(code(typeck::break_outside_loop))]
    BreakOutsideLoop {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("break is outside loop.")]
        span: SourceSpan,
    },
    #[error("could not use continue outside loop.")]
    #[diagnostic(code(typeck::continue_outside_loop))]
    ContinueOutsideLoop {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("continue is outside loop.")]
        span: SourceSpan,
    },
    #[error("unexpected expr in resolution {expr:?}.")]
    #[diagnostic(
        code(typeck::unexpected_expr_in_resolution),
        help("please, file an issue on github."),
        url("https://github.com/wattlanguage/watt")
    )]
    UnexpectedExprInResolution { expr: EcoString },
}

/// Exhaustiveness error
#[derive(Debug, Error, Diagnostic)]
pub enum ExError {
    #[error("enum patterns missmatch.")]
    #[diagnostic(code(ex::enum_patterns_missmatch))]
    EnumPatternsMissmatch {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("patterns are missmatched.")]
        span: SourceSpan,
    },
    #[error("enum fields missmatch.")]
    #[diagnostic(code(ex::enum_unwrap_fields_missmatch))]
    EnumUnwrapFieldsMissmatch {
        #[source_code]
        src: NamedSource<Arc<String>>,
        #[label("fields of patterns are missmatched.")]
        span: SourceSpan,
    },
}
