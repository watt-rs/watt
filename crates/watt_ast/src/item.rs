/// Imports
use crate::{
    atom::{Param, Publicity, TypeHint},
    expr::Expr,
};
use miette::NamedSource;
use std::sync::Arc;
use watt_lex::token::Span;

/// Represents enum varisnt
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Variant {
    pub span: Span,
    pub name: String,
    pub fields: Vec<TypeHint>,
}

/// Import path (e.g `this/is/some/module`)
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ImportPath {
    pub span: Span,
    pub module: String,
}

/// Represents import kind
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ImportKind {
    /// Represents import of module as given name
    As(String),
    /// Represents import of module contents separated by comma
    For(Vec<String>),
    /// Just import of module
    Just,
}

/// Represents import declaration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Import {
    pub span: Span,
    pub path: ImportPath,
    pub kind: ImportKind,
}

/// Represents struct field
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Field {
    pub span: Span,
    pub name: String,
    pub hint: TypeHint,
}

/// Struct item
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Struct {
    pub span: Span,
    pub name: String,
    pub publicity: Publicity,
    pub generics: Vec<String>,
    pub fields: Vec<Field>,
}

/// Enum item
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Enum {
    pub span: Span,
    pub name: String,
    pub publicity: Publicity,
    pub generics: Vec<String>,
    pub variants: Vec<Variant>,
}

/// Function item
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Fn {
    pub span: Span,
    pub publicity: Publicity,
    pub name: String,
    pub generics: Vec<String>,
    pub params: Vec<Param>,
    pub ret: Option<TypeHint>,
    pub block: Expr,
}

/// Extern function item
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExternFn {
    pub span: Span,
    pub name: String,
    pub publicity: Publicity,
    pub generics: Vec<String>,
    pub params: Vec<Param>,
    pub ret: Option<TypeHint>,
    pub body: String,
}

/// Extern type item
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExternType {
    pub span: Span,
    pub name: String,
    pub publicity: Publicity,
    pub generics: Vec<String>,
}

/// Constant item
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Const {
    pub span: Span,
    pub publicity: Publicity,
    pub name: String,
    pub value: Expr,
    pub hint: TypeHint,
}

/// Item declaration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Item {
    Struct(Struct),
    Enum(Enum),
    Fn(Fn),
    ExternFn(ExternFn),
    ExternType(ExternType),
    Const(Const),
}

/// Module item
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Module {
    /// Source of the module
    pub source: Arc<NamedSource<String>>,

    /// Module imports
    pub imports: Vec<Import>,

    /// Module items
    pub items: Vec<Item>,
}
