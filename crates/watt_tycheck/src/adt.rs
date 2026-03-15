/// Imports
use crate::typ::Typ;
use watt_lex::token::Span;

/// Represents a field of a struct.
/// ! Contains uninstantiated type !
#[derive(Clone, PartialEq, Debug)]
pub struct Field {
    pub name: String,
    pub span: Span,
    pub typ: Typ,
}

/// Represents structure type scheme
#[derive(Clone)]
pub struct Struct {
    pub span: Span,
    pub name: String,
    pub generics: Vec<String>,
    pub fields: Vec<Field>,
}

/// Represents enum variant
/// ! Contains uninstantiated type !
#[derive(Clone, PartialEq)]
pub struct EnumVariant {
    pub span: Span,
    pub name: String,
    pub fields: Vec<Typ>,
}
