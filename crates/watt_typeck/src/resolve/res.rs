/// Imports
use crate::{
    errors::TypeckError,
    typ::{CustomType, Enum, EnumVariant, Typ},
};
use ecow::EcoString;
use std::rc::Rc;
use watt_common::{address::Address, bail};

// Resolution
#[derive(Debug, Clone)]
pub enum Res {
    Module(EcoString),
    Custom(CustomType),
    Variant(Rc<Enum>, EnumVariant),
    Value(Typ),
}

/// Implementation
impl Res {
    /// Unwraps resolution as typ,
    /// if resolution isn't typ, raises error
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
