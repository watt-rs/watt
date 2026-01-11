/// Imports
use crate::{
    errors::{TypeckError, TypeckRelated},
    inference::coercion::CoercionError,
};
use watt_common::{address::Address, bail};

/// A cause describing *why* a type constraint was introduced.
///
/// `Cause` is used exclusively for diagnostics. It does **not** affect
/// type inference or unification logic, but is attached to constraints
/// in order to produce precise and user-friendly error messages.
///
/// In other words, it answers the question:
/// > "In which language construct did this type check occur?"
///
/// This design mirrors the approach used in `rustc`, where the
/// unification engine is context-agnostic, while error reporting
/// is enriched using an explicit cause stack.
///
#[derive(Debug, Clone)]
pub enum Cause<'a> {
    /// A type constraint originating from a function call argument.
    ///
    /// Example:
    /// ```watt
    /// foo(1, true)
    ///        ^^^^
    /// ```
    ///
    FunctionArgument(&'a Address),

    /// A type constraint originating from a struct creation argument.
    ///
    /// Example:
    /// ```watt
    /// Struct(arg1, arg2)
    ///              ^^^^
    /// ```
    ///
    StructArgument(&'a Address),

    /// A type constraint originating from a variant creation argument.
    ///
    /// Example:
    /// ```watt
    /// Enum.Variant(arg1, arg2)
    ///              ^^^^
    /// ```
    ///
    VariantArgument(&'a Address),

    /// A type constraint originating from an assignment expression.
    ///
    /// Example:
    /// ```watt
    /// let x: int = expr;
    /// ```
    ///
    Assignment(&'a Address),

    /// A type constraint originating from an return type and block type match check.
    ///
    /// Example:
    /// ```watt
    /// fn a(): int {
    ///     325
    /// }
    /// ```
    ///
    Return(&'a Address, &'a Address),

    /// A type constraint originating from an pattern type and matchable type match check.
    ///
    /// Example:
    /// ```watt
    /// match option {
    ///       ^^^^^^
    ///     Option.Some(value) -> value,
    ///     ^^^^^^^^^^^^^^^^^
    ///     Option.None -> todo
    /// }
    /// ```
    ///
    Pattern(&'a Address, &'a Address),

    /// A type constraint originating from an branch types match check.
    ///
    /// Example:
    /// ```watt
    /// match option {
    /// ^^^^^
    ///     Option.Some(value) -> value,
    ///                           ^^^^^
    ///     Option.None -> todo
    /// }
    /// ```
    ///
    Branch(&'a Address, &'a Address),
}

/// Implementation of the cause
impl<'a> Cause<'a> {
    /// Converts a `Cause` and a `CoercionError` into a `TypeckError`.
    ///
    /// This is used in the type-checking phase to generate detailed
    /// error reports including the relevant source spans and the types involved.
    ///
    /// # Parameters
    /// - `self` — the cause of the type error, e.g., function argument, assignment, pattern, etc.
    /// - `error` — the specific coercion error that occurred (e.g., recursive types or mismatch).
    /// - `p1` — the first type involved (expected type or one side of recursion).
    /// - `p2` — the second type involved (actual type or other side of recursion).
    ///
    /// # Returns
    /// A `TypeckError` containing detailed information about the error, including
    /// related source spans and types, suitable for reporting to the user.
    ///
    /// # Behavior
    /// - If the `CoercionError` is `RecursiveType`, the method generates
    ///   a `TypeckError::RecursiveType` with type and source spab.
    /// - If the `CoercionError` is `TypesMissmatch`, the method generates
    ///   a `TypeckError::TypesMissmatch`, adjusting the related spans depending
    ///   on the specific `Cause` variant (e.g., assignment, function return, pattern, branch).
    ///
    pub(crate) fn into_typeck_error(
        self,
        error: CoercionError,
        p1: String,
        p2: String,
    ) -> TypeckError {
        match error {
            CoercionError::RecursiveType => match self {
                Cause::StructArgument(address)
                | Cause::VariantArgument(address)
                | Cause::FunctionArgument(address)
                | Cause::Assignment(address)
                | Cause::Return(address, _)
                | Cause::Pattern(address, _)
                | Cause::Branch(address, _) => bail!(TypeckError::RecursiveType {
                    related: vec![TypeckRelated::Here {
                        src: address.source.clone(),
                        span: address.span.clone().into()
                    }],
                    t: p1
                }),
            },
            CoercionError::TypesMissmatch => match self {
                Cause::StructArgument(address)
                | Cause::VariantArgument(address)
                | Cause::FunctionArgument(address)
                | Cause::Assignment(address) => bail!(TypeckError::TypesMissmatch {
                    related: vec![TypeckRelated::Here {
                        src: address.source.clone(),
                        span: address.span.clone().into()
                    }],
                    expected: p1,
                    got: p2
                }),
                Cause::Return(a1, a2) => {
                    bail!(TypeckError::TypesMissmatch {
                        related: vec![
                            TypeckRelated::ThisType {
                                src: a1.source.clone(),
                                span: a1.span.clone().into(),
                                t: p1.clone()
                            },
                            TypeckRelated::ThisType {
                                src: a2.source.clone(),
                                span: a2.span.clone().into(),
                                t: p2.clone()
                            }
                        ],
                        expected: p1,
                        got: p2
                    })
                }
                Cause::Pattern(a1, a2) => {
                    bail!(TypeckError::TypesMissmatch {
                        related: vec![
                            TypeckRelated::ThisType {
                                src: a1.source.clone(),
                                span: a1.span.clone().into(),
                                t: p1.clone()
                            },
                            TypeckRelated::ThisType {
                                src: a2.source.clone(),
                                span: a2.span.clone().into(),
                                t: p2.clone()
                            }
                        ],
                        expected: p1,
                        got: p2
                    })
                }
                Cause::Branch(a1, a2) => {
                    bail!(TypeckError::TypesMissmatch {
                        related: vec![
                            TypeckRelated::ThisType {
                                src: a1.source.clone(),
                                span: a1.span.clone().into(),
                                t: p1.clone()
                            },
                            TypeckRelated::ThisType {
                                src: a2.source.clone(),
                                span: a2.span.clone().into(),
                                t: p2.clone()
                            }
                        ],
                        expected: p1,
                        got: p2
                    })
                }
            },
        }
    }
}
