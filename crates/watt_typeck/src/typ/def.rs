/// Imports
use crate::typ::typ::{Enum, Function, Struct, Typ, WithPublicity};
use std::{cell::RefCell, fmt::Debug, rc::Rc};

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
///   Represents a constant variable with a fully inferenced type.
///   The type of a constant **cannot** be `Typ::Generic` or `Typ::Unbound`; it must
///   be fully concrete (`Prelude`, `Struct`, `Enum`, `Function`, or `Unit`).
///
/// - `Function(WithPublicity<Rc<Function>>)`
///   Represents a function.
///
#[derive(Clone)]
pub enum ModuleDef {
    /// User-defined type
    Type(WithPublicity<TypeDef>),
    /// Function
    Function(WithPublicity<Rc<Function>>),
    /// Constant with fully inferred type
    Const(WithPublicity<Typ>),
}

/// Debug implementation
impl Debug for ModuleDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModuleDef::Type(ty) => write!(f, "Type({ty:?})"),
            ModuleDef::Const(ty) => write!(f, "Const({ty:?})"),
            ModuleDef::Function(ty) => write!(f, "Function({ty:?})"),
        }
    }
}

/// Represents a type definition for a resolver
///
/// # Variants
///
/// - `Enum(Rc<RefCell<Enum>>)`
///   Represents enum type definition
///
/// - `Struct(Rc<RefCell<Struct>>)`
///   Represents struct type definition
///
#[derive(Clone, PartialEq)]
pub enum TypeDef {
    Enum(Rc<RefCell<Enum>>),
    Struct(Rc<RefCell<Struct>>),
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
