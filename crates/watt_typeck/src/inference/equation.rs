/// Imports
use crate::typ::typ::Typ;
use watt_common::address::Address;

/// Represents a single inference unit
///
/// `Unit` is a lightweight structure used during **type inference** and **unification**
/// to associate a type (`Typ`) with a specific source code location (`Address`).
///
/// # Fields
///
/// 1) `Address`
///    The exact position in the source code where this variable or type expression
///    originates. This allows precise error reporting when type mismatches occur.
///
/// 1) `Typ`
///    The actual type (or type expression) being unified or constrained.
///
pub type Unit = (Address, Typ);

/// Represents a single type equation used in the unification process.
///
/// The `Equation` enum defines constraints between types that must be unified.
/// It is a central part of the **type inference** or **type checking** phase,
/// where the compiler ensures type consistency across expressions.
///
/// # Variants
///
/// - `Unify(Var, Var)`
///   Represents a unification equation between two type variables.
///   Example: `a == b`, meaning both must resolve to the same type.
///
/// - `UnifyMany(Vec<Var>)`
///   Represents a generalized unification between multiple variables.
///   All variables in the list must share a common type after unification.
///
/// # Example
///
/// ```rust
/// // Example: a simple types unification
/// let eq = Equation::Unify(var_a, var_b);
///
/// // Example: unify multiple types together
/// let eq_many = Equation::UnifyMany(vec![var_x, var_y, var_z]);
/// ```
///
#[derive(Debug, Clone)]
pub enum Equation {
    /// Unifies two types.
    Unify(Unit, Unit),

    /// Unifies a group of types, ensuring that all group types
    /// are the same or all group types have the same basic type.
    UnifyMany(Vec<Unit>),
}
