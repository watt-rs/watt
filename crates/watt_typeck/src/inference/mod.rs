/// Modules
pub mod coercion;
pub mod generics;
pub mod hydrator;
pub mod origin;

/// Imports
use crate::{
    errors::TypeckError,
    inference::{
        coercion::{Coercion, U},
        hydrator::Hydrator,
        origin::Origin,
    },
    typ::typ::Typ,
};
use tracing::instrument;
use watt_common::bail;

/// `CoercionsSolver` — a solver for type coercions in the type inference system.
///
/// It works with `Coercion` variants:
/// - `Eq(u1, u2)` — unifies two types `u1` and `u2`
/// - `Same(items)` — unifies all types in the list `items`
///
/// Uses `Hydrator` to store and apply type substitutions.
///
/// Example usage:
/// ```ignore
/// let mut solver = CoercionsSolver::default();
/// solver.coerce(Coercion::Eq(u1, u2));
/// ```
///
#[derive(Default, Debug)]
pub struct CoercionsSolver {
    /// Module hydrator
    pub(crate) hydrator: Hydrator,
}

/// Implementation
impl CoercionsSolver {
    /// Solves a coercion, dispatching to `eq` or `same`.
    ///
    /// # Arguments
    /// * `coercion` - the type constraint to solve
    ///
    pub fn coerce(&mut self, coercion: Coercion) {
        // Solving
        match coercion.clone() {
            Coercion::Eq(u1, u2) => self.eq(u1, u2),
            Coercion::Same(items) => self.same(items),
        }
    }

    /// Solves an `Eq(u1, u2)` coercion.
    #[instrument(skip(self), level = "debug", fields(u1 = ?u1.1, u2 = ?u2.1))]
    fn eq(&mut self, u1: U, u2: U) {
        // Generation origins
        let o1 = Origin(u1.0, u1.1.clone());
        let o2 = Origin(u2.0, u2.1.clone());
        // Processing unification
        self.unify(o1, u1.1, o2, u2.1);
    }

    /// Solves a `Same(items)` coercion,
    /// unifying all elements with the first one.
    #[instrument(
        skip(self),
        level = "debug",
        fields(items = ?items.iter().map(|i| &i.1).collect::<Vec<&Typ>>()))
    ]
    fn same(&mut self, mut items: Vec<U>) {
        // Retrieving first information
        let u1 = items.remove(0);
        // Coerce `eq` with others
        for u2 in items {
            self.eq(u1.clone(), u2);
        }
    }

    /// Core method to unify two types.
    ///
    /// Performs occurs check in case with `Unbound(a)` and other `b`.
    /// Calls `Hydrator` for substitutions.
    ///
    fn unify(&mut self, o1: Origin, t1: Typ, o2: Origin, t2: Typ) {
        // Applying substs
        let t1 = self.apply(t1);
        let t2 = self.apply(t2);
        // Unifying
        if t1 != t2 {
            match (&t1, &t2) {
                (Typ::Unbound(a), Typ::Unbound(b)) => {
                    if a != b {
                        self.hydrator.substitute(*a, t2.clone());
                    }
                }
                (Typ::Unbound(a), b) | (b, Typ::Unbound(a)) => {
                    if self.occurs(*a, b) {
                        bail!(TypeckError::TypesRecursion {
                            related: vec![o1.into_this_type(), o2.into_this_type(),],
                            t1: o1.typ(),
                            t2: o2.typ()
                        })
                    }
                    self.hydrator.substitute(*a, b.clone());
                }
                (Typ::Struct(def1, _), Typ::Struct(def2, _)) => {
                    if def1 == def2 {
                        t1.fields(&mut self.hydrator)
                            .into_iter()
                            .zip(t2.fields(&mut self.hydrator))
                            .for_each(|(a, b)| {
                                self.unify(o1.clone(), a.typ, o2.clone(), b.typ);
                            });
                    }
                }
                (Typ::Enum(def1, _), Typ::Enum(def2, _)) => {
                    if def1 == def2 {
                        t1.variants(&mut self.hydrator)
                            .iter()
                            .zip(t2.variants(&mut self.hydrator))
                            .for_each(|(v1, v2)| {
                                v1.fields.iter().zip(v2.fields).for_each(|(a, b)| {
                                    self.unify(
                                        o1.clone(),
                                        a.typ.clone(),
                                        o2.clone(),
                                        b.typ.clone(),
                                    );
                                });
                            });
                    }
                }
                (Typ::Function(f1), Typ::Function(f2)) => {
                    f1.params.iter().zip(&f2.params).for_each(|(p1, p2)| {
                        self.unify(o1.clone(), p1.typ.clone(), o2.clone(), p2.typ.clone());
                    });
                    self.unify(o1.clone(), f1.ret.clone(), o2.clone(), f2.ret.clone());
                }
                _ => bail!(TypeckError::CouldNotUnify {
                    t1: o1.typ(),
                    t2: o2.typ(),
                    related: vec![o1.into_this_type(), o2.into_this_type(),]
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
    fn occurs(&mut self, own: usize, t: &Typ) -> bool {
        let t = self.apply(t.clone());

        match t {
            Typ::Unbound(id) => {
                // variable occurs in itself → infinite type
                id == own
            }
            Typ::Function(ref fun) => {
                fun.params.iter().any(|p| self.occurs(own, &p.typ)) || self.occurs(own, &fun.ret)
            }
            it @ Typ::Struct(_, _) => it
                .fields(&mut self.hydrator)
                .into_iter()
                .any(|f| self.occurs(own, &f.typ)),
            it @ Typ::Enum(_, _) => it
                .variants(&mut self.hydrator)
                .iter()
                .any(|v| v.fields.iter().any(|f| self.occurs(own, &f.typ))),
            Typ::Generic(_) | Typ::Prelude(_) | Typ::Unit => false,
        }
    }

    /// Applies substitutions to type
    pub fn apply(&mut self, typ: Typ) -> Typ {
        self.hydrator.apply(typ)
    }
}
