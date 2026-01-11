/// Import
use crate::{
    errors::TypeckError,
    inference::origin::Origin,
    typ::{
        cx::InferCx,
        typ::{TyVar, Typ},
    },
};
use id_arena::Id;
use tracing::instrument;
use watt_common::{address::Address, bail};

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

/// Solves a coercion, dispatching to `eq` or `same`.
///
/// # Arguments
/// * `coercion` - the type constraint to solve
///
pub fn coerce(icx: &mut InferCx, coercion: Coercion) {
    // Solving
    match coercion.clone() {
        Coercion::Eq(u1, u2) => eq(icx, u1, u2),
        Coercion::Same(items) => same(icx, items),
    }
}

/// Solves an `Eq(u1, u2)` coercion.
#[instrument(skip(icx), level = "trace", fields(u1 = ?u1.1, u2 = ?u2.1))]
fn eq(icx: &mut InferCx, u1: U, u2: U) {
    // Generation origins
    let o1 = Origin(u1.0, u1.1.clone());
    let o2 = Origin(u2.0, u2.1.clone());
    // Processing unification
    unify(icx, o1, u1.1, o2, u2.1);
}

/// Solves a `Same(items)` coercion,
/// unifying all elements with the first one.
#[instrument(
    skip(icx),
    level = "trace",
    fields(items = ?items.iter().map(|i| &i.1).collect::<Vec<&Typ>>()))
]
fn same(icx: &mut InferCx, mut items: Vec<U>) {
    // Retrieving first information
    let u1 = items.remove(0);
    // Coerce `eq` with others
    for u2 in items {
        eq(icx, u1.clone(), u2);
    }
}

/// Core method to unify two types.
///
/// Performs occurs check in case with `Unbound(a)` and other `b`.
/// Calls `Hydrator` for substitutions.
///
fn unify(icx: &mut InferCx, o1: Origin, t1: Typ, o2: Origin, t2: Typ) {
    // Applying substs
    let t1 = icx.apply(t1);
    let t2 = icx.apply(t2);
    // Unifying
    if t1 != t2 {
        match (&t1, &t2) {
            (Typ::Var(a), Typ::Var(b)) => {
                if a != b {
                    icx.substitute(*a, t2.clone());
                }
            }
            (Typ::Var(a), b) | (b, Typ::Var(a)) => {
                if occurs(icx, *a, b) {
                    bail!(TypeckError::TypesRecursion {
                        related: vec![o1.into_this_type(icx), o2.into_this_type(icx)],
                        t1: o1.typ().pretty(icx),
                        t2: o2.typ().pretty(icx)
                    })
                }
                icx.substitute(*a, b.clone());
            }
            (Typ::Struct(id1, _), Typ::Struct(id2, _)) => {
                if id1 == id2 {
                    t1.fields(icx)
                        .into_iter()
                        .zip(t2.fields(icx))
                        .for_each(|(a, b)| {
                            unify(icx, o1.clone(), a.typ.clone(), o2.clone(), b.typ.clone());
                        });
                } else {
                    bail!(TypeckError::CouldNotUnify {
                        related: vec![o1.into_this_type(icx), o2.into_this_type(icx)],
                        t1: o1.1.pretty(icx),
                        t2: o2.1.pretty(icx),
                    })
                }
            }
            (Typ::Enum(def1, _), Typ::Enum(def2, _)) => {
                if def1 == def2 {
                    t1.variants(icx)
                        .into_iter()
                        .zip(t2.variants(icx))
                        .for_each(|(v1, v2)| {
                            v1.fields.iter().zip(v2.fields).for_each(|(a, b)| {
                                unify(icx, o1.clone(), a.typ.clone(), o2.clone(), b.typ.clone());
                            });
                        });
                } else {
                    bail!(TypeckError::CouldNotUnify {
                        related: vec![o1.into_this_type(icx), o2.into_this_type(icx)],
                        t1: o1.1.pretty(icx),
                        t2: o2.1.pretty(icx),
                    })
                }
            }
            (Typ::Function(_, _), Typ::Function(_, _)) => {
                t1.params(icx)
                    .into_iter()
                    .zip(t2.params(icx))
                    .for_each(|(p1, p2)| {
                        unify(icx, o1.clone(), p1.typ.clone(), o2.clone(), p2.typ.clone());
                    });
                let r1 = t1.ret(icx);
                let r2 = t2.ret(icx);
                unify(icx, o1.clone(), r1, o2.clone(), r2);
            }
            _ => bail!(TypeckError::CouldNotUnify {
                related: vec![o1.into_this_type(icx), o2.into_this_type(icx)],
                t1: o1.1.pretty(icx),
                t2: o2.1.pretty(icx),
            }),
        }
    }
}

/// Occurs check — ensures that a type variable does not appear within itself.
///
/// # Arguments
/// * `own` — the type variable identifier
/// * `t` — the type to check for occurrence
///
/// # Returns
/// `true` if the type variable occurs in itself (infinite type), otherwise `false`
///
fn occurs(icx: &mut InferCx, own: Id<TyVar>, t: &Typ) -> bool {
    let t = icx.apply(t.clone());

    match t {
        Typ::Var(id) => {
            // variable occurs in itself → infinite type
            id == own
        }
        it @ Typ::Function(_, _) => {
            it.params(icx).into_iter().any(|p| occurs(icx, own, &p.typ)) || {
                let r = it.ret(icx);
                occurs(icx, own, &r)
            }
        }
        it @ Typ::Struct(_, _) => it.fields(icx).into_iter().any(|f| occurs(icx, own, &f.typ)),
        it @ Typ::Enum(_, _) => it
            .variants(icx)
            .iter()
            .any(|v| v.fields.iter().any(|f| occurs(icx, own, &f.typ))),
        Typ::Generic(_) | Typ::Prelude(_) | Typ::Unit => false,
    }
}
