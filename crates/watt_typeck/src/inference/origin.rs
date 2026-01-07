/// Imports
use crate::{
    errors::TypeckRelated,
    typ::{cx::InferCx, typ::Typ},
};
use watt_common::address::Address;

/// Represents the origin of a type in the unification process.
///
/// `Origin` bundles the source code location together with the (possibly
/// partially inferred) type to produce meaningful diagnostics.
#[derive(Clone, Debug)]
pub struct Origin(pub Address, pub Typ);

/// Implementation
#[allow(clippy::wrong_self_convention)]
impl Origin {
    /// Converts this origin into a `TypeckRelated::ThisType`,
    /// preserving source information and attaching the type.
    pub fn into_this_type(&self, icx: &mut InferCx) -> TypeckRelated {
        TypeckRelated::ThisType {
            src: self.0.source.clone(),
            span: self.0.span.clone().into(),
            t: self.1.pretty(icx),
        }
    }

    /// Converts this origin into `TypeckRelated::This`,
    /// preserving only source location.
    pub fn into_this(&self) -> TypeckRelated {
        TypeckRelated::This {
            src: self.0.source.clone(),
            span: self.0.span.clone().into(),
        }
    }

    /// Converts this origin into `TypeckRelated::WithThis`,
    /// used when additional context should be associated with diagnostics.
    pub fn into_with_this(&self) -> TypeckRelated {
        TypeckRelated::WithThis {
            src: self.0.source.clone(),
            span: self.0.span.clone().into(),
        }
    }

    /// Returns the type associated with this origin.
    #[inline]
    pub fn typ(&self) -> Typ {
        self.1.clone()
    }
}
