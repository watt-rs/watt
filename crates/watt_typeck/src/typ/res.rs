/// Imports
use crate::{
    errors::TypeckError,
    pretty::Pretty,
    typ::{
        cx::InferCx,
        def::TypeDef,
        typ::{EnumVariant, Typ},
    },
};
use ecow::EcoString;
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
/// - `Variant(Typ, EnumVariant)`
///   The identifier resolves to a specific enum variant. Stores a reference
///   to the enum instance and the resolved variant.
///
/// - `Value(Typ)`
///   The identifier resolves directly to a type/value
///
/// - `Const(Typ)`
///   The identifier resolves directly to a const type/value
///
#[derive(Debug, Clone)]
pub enum Res {
    Module(EcoString),
    Custom(TypeDef),
    Variant(Typ, EnumVariant),
    Value(Typ),
    Const(Typ),
}

/// Resolution implementation
impl Res {
    /// Unwraps the resolution as a concrete type (`Typ`).
    ///
    /// If the resolution is not a type (e.g., it's a module or enum variant),
    /// this function will raise a type checking error.
    ///
    /// # Arguments
    /// - `icx: &mut InferCx` – Represents infrence context, used for pretty-printing.
    /// - `address: &Address` – The source code location used for error reporting.
    ///
    /// # Returns
    /// - `Typ` – The concrete type resolved.
    ///
    /// # Errors
    /// - [`TypeckError::UnexpectedResolution`]: if the resolution
    ///   is not a `Res::Value` or `Res::Const`.
    /// 
    pub fn unwrap_typ(self, icx: &mut InferCx, address: &Address) -> Typ {
        match self {
            Res::Value(t) => t,
            Res::Const(t) => t,
            _ => bail!(TypeckError::UnexpectedResolution {
                src: address.source.clone(),
                span: address.clone().span.into(),
                res: self.pretty(icx),
            }),
        }
    }
}

/// Pretty implementation
impl Pretty for Res {
    /// Pretty prints resolution
    ///
    /// # Parameters
    /// - `icx: &mut InferCx`
    ///   Inference context used
    ///   to pretty print types
    ///
    fn pretty(&self, icx: &mut InferCx) -> String {
        // Matching self
        match self {
            Res::Module(name) => format!("Module({name})"),
            Res::Custom(def) => def.pretty(icx),
            Res::Variant(typ, variant) => format!("Variant({}.{})", typ.pretty(icx), variant.name),
            Res::Value(typ) => format!("Value({})", typ.pretty(icx)),
            Res::Const(typ) => format!("Const({})", typ.pretty(icx)),
        }
    }
}
