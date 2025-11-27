/// Imports
use crate::typ::typ::Typ;
use watt_common::address::Address;
use crate::errors::TypeckRelated;

/// Represents a single inference unit.
///
/// `Unit` is a compact structure used during **type inference** and **unification**.
/// It associates a type (`Typ`) with a concrete source code location (`Address`),
/// allowing precise reporting of type mismatches.
///
/// # Fields
/// - `Address` — location in the source code where this type originated.
/// - `Typ` — the inferred or expected type associated with that location.
///
#[derive(Debug, Clone)]
pub struct EqUnit(
    pub Address,
    pub Typ
);

impl EqUnit {
    /// Creates a new [`Unit`] from a source location and a type.
    #[inline]
    pub fn new(address: Address, typ: Typ) -> Self {
        Self(address, typ)
    }
}

/// Represents the origin of a type in the inference process.
///
/// `Origin` bundles the source code location together with the (possibly
/// partially inferred) type to produce meaningful diagnostics.
#[derive(Clone, Debug)]
pub struct Origin(
    pub Address,
    pub Typ
);

#[allow(clippy::wrong_self_convention)]
impl Origin {
    /// Converts this origin into a `TypeckRelated::ThisType`,
    /// preserving source information and attaching the type.
    pub fn into_this_type(&self) -> TypeckRelated {
        TypeckRelated::ThisType {
            src: self.0.source.clone(),
            span: self.0.span.clone().into(),
            t: self.1.clone(),
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

/// A constraint in the type inference system.
///
/// `Equation` represents a relationship between one or more types that must be
/// resolved during unification. These constraints are processed by the type
/// checker to ensure consistency of type expressions across the program.
///
/// # Variants
///
/// - [`Unify`] — unifies two types or type variables.
/// - [`UnifyMany`] — unifies all types within a group, requiring them to resolve
///   to a single consistent type.
///
/// # Examples
/// ```rust
/// // Unify two types
/// let eq = Equation::Unify(unit_a, unit_b);
///
/// // Unify a list of types
/// let eq_many = Equation::UnifyMany(vec![unit_x, unit_y, unit_z]);
/// ```
#[derive(Debug, Clone)]
pub enum Equation {
    /// A binary unification constraint between two types.
    Unify(EqUnit, EqUnit),

    /// A variadic unification constraint: all types in the list must
    /// unify to the same resulting type.
    UnifyMany(Vec<EqUnit>),
}
