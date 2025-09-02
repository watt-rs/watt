/// Imports
use crate::analyze::{
    errors::AnalyzeError,
    rc_ptr::RcPtr,
    typ::{CustomType, Enum, EnumVariant, Typ},
};
use ecow::EcoString;
use miette::NamedSource;
use oil_common::{address::Address, bail};
use std::sync::Arc;

// Resolution
#[derive(Debug)]
pub enum Res {
    Module(EcoString),
    Custom(CustomType),
    Variant(RcPtr<Enum>, EnumVariant),
    Value(Typ),
}

/// Implementation
impl Res {
    /// Unwraps resolution as typ,
    /// if resolution isn't typ, raises error
    pub fn unwrap_typ(self, source: &NamedSource<Arc<String>>, address: &Address) -> Typ {
        match self {
            Res::Value(t) => t,
            _ => bail!(AnalyzeError::UnexpectedResolution {
                src: source.clone(),
                span: address.clone().span.into(),
                res: self
            }),
        }
    }
}
