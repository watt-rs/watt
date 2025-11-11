/// Imports
use crate::typ::typ::{Enum, Struct, Typ, WithPublicity};
use std::{fmt::Debug, rc::Rc};

/// Represents a module definition for a resolver.
///
/// `ModuleDef` describes items that can exist inside a module, such as user-defined
/// types or constants.
///
/// # Variants
///
/// - `Type(WithPublicity<TypeDef>)`
///   Represents a user-defined type. Prelude or primitive types aren't stored here.
///   Contains information about generics, fields, etc.
///
/// - `Const(WithPublicity<Typ>)`
///   Represents a constant variable with a fully inferred type.
///   The type of a constant **cannot** be `Typ::Generic` or `Typ::Unbound`; it must
///   be fully concrete (`Prelude`, `Struct`, `Enum`, `Function`, or `Unit`).
///
#[derive(Clone)]
pub enum ModuleDef {
    /// User-defined type
    Type(WithPublicity<TypeDef>),
    /// Constant with fully inferred type
    Const(WithPublicity<Typ>),
}

/// Represents a type definition for a resolver
///
/// # Variants
///
/// - `Enum(Rc<Enum>)`
///   Represents enum type definition
///
/// - `Struct(Rc<Struct>)`
///   Represents struct type definition
///
#[derive(Clone, PartialEq)]
pub enum TypeDef {
    Enum(Rc<Enum>),
    Struct(Rc<Struct>),
}

/// Debug implementation
impl Debug for TypeDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeDef::Enum(en) => write!(f, "Enum({en:?})"),
            TypeDef::Struct(ty) => write!(f, "Struct({ty:?})"),
        }
    }
}
