/// Imports
use watt_common::{address::Address, bail};

use crate::{
    errors::{TypeckError, TypeckRelated},
    inference::coercion::CoercionError,
};

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
    // Transform cause into `TypeckError`
    pub(crate) fn into_typeck_error(
        self,
        error: CoercionError,
        p1: String,
        p2: String,
    ) -> TypeckError {
        match error {
            CoercionError::TypesRecursion => match self {
                Cause::StructArgument(address)
                | Cause::VariantArgument(address)
                | Cause::FunctionArgument(address)
                | Cause::Assignment(address)
                | Cause::Return(address, _)
                | Cause::Pattern(address, _)
                | Cause::Branch(address, _) => bail!(TypeckError::TypesRecursion {
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
                Cause::Return(body, ret) => {
                    bail!(TypeckError::TypesMissmatch {
                        related: vec![
                            TypeckRelated::ThisType {
                                src: body.source.clone(),
                                span: body.span.clone().into(),
                                t: p1.clone()
                            },
                            TypeckRelated::ThisType {
                                src: ret.source.clone(),
                                span: ret.span.clone().into(),
                                t: p2.clone()
                            }
                        ],
                        expected: p1,
                        got: p2
                    })
                }
                Cause::Pattern(matchable, pattern) => {
                    bail!(TypeckError::TypesMissmatch {
                        related: vec![
                            TypeckRelated::ThisType {
                                src: matchable.source.clone(),
                                span: matchable.span.clone().into(),
                                t: p1.clone()
                            },
                            TypeckRelated::ThisType {
                                src: pattern.source.clone(),
                                span: pattern.span.clone().into(),
                                t: p2.clone()
                            }
                        ],
                        expected: p1,
                        got: p2
                    })
                }
                Cause::Branch(match_, case) => {
                    bail!(TypeckError::TypesMissmatch {
                        related: vec![
                            TypeckRelated::ThisType {
                                src: match_.source.clone(),
                                span: match_.span.clone().into(),
                                t: p1.clone()
                            },
                            TypeckRelated::ThisType {
                                src: case.source.clone(),
                                span: case.span.clone().into(),
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
