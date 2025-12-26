/// Import
use crate::typ::typ::Typ;
use watt_common::address::Address;

/// A constraint unit, contains type and address.
pub type U = (Address, Typ);

/// A constraint in the type inference system.
///
/// `Coercion` represents a relationship between one or more types that must be
/// resolved during unification. These constraints are processed by the type
/// checker to ensure consistency of type expressions across the program.
///
/// # Variants
///
/// - [`Eq`] — unifies two types or type variables.
/// - [`Same`] — unifies all types within a group are same.
///
#[derive(Debug, Clone)]
pub enum Coercion {
    /// Equality of two types
    Eq(U, U),
    /// Equality of many types
    Same(Vec<U>),
}
