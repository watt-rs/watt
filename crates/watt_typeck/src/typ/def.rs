/// Imports
use crate::{
    pretty::Pretty,
    typ::{
        cx::InferCx,
        typ::{Enum, Function, Struct, Typ, WithPublicity},
    },
};
use id_arena::Id;
use std::fmt::Debug;

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
///   Represents a constant variable with a fully infered type.
///   The type of constant **cannot** be `Typ::Generic` or `Typ::Unbound`; it must
///   be fully concrete (`Prelude`, `Struct`, `Enum`, `Function`, or `Unit`).
///
/// - `Function(WithPublicity<Rc<Function>>)`
///   Represents a function.
///
#[derive(Clone, Debug)]
pub enum ModuleDef {
    /// User-defined type
    Type(WithPublicity<TypeDef>),
    /// Function
    Function(WithPublicity<Id<Function>>),
    /// Constant with fully inferred type
    Const(WithPublicity<Typ>),
}

/// Pretty implementation for `ModuleDef`
impl Pretty for ModuleDef {
    fn pretty(&self, icx: &mut InferCx) -> String {
        match self {
            ModuleDef::Type(ty) => ty.value.pretty(icx),
            ModuleDef::Function(f) => format!("Function({})", icx.tcx.function(f.value).name),
            ModuleDef::Const(ty) => format!("Const({})", ty.value.pretty(icx)),
        }
    }
}

/// Represents a type definition for a resolver
///
/// # Variants
///
/// - `Enum(Id<Enum>)`
///   Represents enum type definition
///
/// - `Struct(Id<Struct>)`
///   Represents struct type definition
///
#[derive(Clone, PartialEq)]
pub enum TypeDef {
    Enum(Id<Enum>),
    Struct(Id<Struct>),
}

/// Pretty implementation for `TypeDef`
impl Pretty for TypeDef {
    fn pretty(&self, icx: &mut InferCx) -> String {
        match self {
            TypeDef::Enum(id) => format!("Enum({})", icx.tcx.enum_(*id).name),
            TypeDef::Struct(id) => format!("Struct({})", icx.tcx.struct_(*id).name),
        }
    }
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
