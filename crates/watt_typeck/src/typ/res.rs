/// Imports
use crate::{
    errors::TypeckError,
    typ::{
        def::TypeDef,
        typ::{Enum, EnumVariant, Typ},
    },
};
use ecow::EcoString;
use std::rc::Rc;
use watt_common::{address::Address, bail};

/// Represents the result of a name/type resolution.
///
/// `Res` is used during type checking and name resolution to indicate
/// what a given identifier resolves to. This could be a module, a
/// custom type, an enum variant, or a concrete value/type.
///
/// # Variants
///
/// - `Module(EcoString)`
///   The identifier resolves to a module. The `EcoString` is the module name.
///
/// - `Custom(TypeDef)`
///   The identifier resolves to a user-defined type (struct or enum).
///
/// - `Variant(Rc<Enum>, EnumVariant)`
///   The identifier resolves to a specific enum variant. Stores a reference
///   to the enum and the resolved variant.
///
/// - `Value(Typ)`
///   The identifier resolves directly to a type/value, which can be any
///   concrete `Typ` (including prelude types, structs, enums, functions, or unit).
///
#[derive(Debug, Clone)]
pub enum Res {
    Module(EcoString),
    Custom(TypeDef),
    Variant(Rc<Enum>, EnumVariant),
    Value(Typ),
}

/// Resolution implementation
impl Res {
    /// Unwraps the resolution as a concrete type (`Typ`).
    ///
    /// If the resolution is not a type (e.g., it's a module or enum variant),
    /// this function will raise a type checking error.
    ///
    /// # Arguments
    ///
    /// * `address: &Address` – The source code location used for error reporting.
    ///
    /// # Returns
    ///
    /// * `Typ` – The concrete type resolved.
    ///
    /// # Panics / Errors
    ///
    /// Raises `TypeckError::UnexpectedResolution` if the resolution
    /// is not a `Res::Value`.
    pub fn unwrap_typ(self, address: &Address) -> Typ {
        match self {
            Res::Value(t) => t,
            _ => bail!(TypeckError::UnexpectedResolution {
                src: address.source.clone(),
                span: address.clone().span.into(),
                res: self
            }),
        }
    }
}
