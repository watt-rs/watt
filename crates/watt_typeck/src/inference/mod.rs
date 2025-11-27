/// Modules
pub mod equation;
pub mod generics;
pub mod hydrator;

/// Imports
use crate::{
    errors::{TypeckError, TypeckRelated},
    inference::{
        equation::{Equation, EqUnit},
        hydrator::Hydrator,
    },
    typ::typ::{PreludeType, Typ},
};
use log::trace;
use watt_common::{address::Address, bail};
use crate::inference::equation::Origin;

/// Equations solver
#[derive(Default)]
pub struct EquationsSolver {
    /// Module hydrator
    pub(crate) hydrator: Hydrator,
}

/// Implementation
impl EquationsSolver {
    /// Solves the equation
    pub fn solve(&mut self, equation: Equation) -> Typ {
        trace!("solving equation: {equation:?}");
        // Solving
        match equation.clone() {
            Equation::Unify(v1, v2) => {
                let o1 = Origin(v1.0, v1.1.clone());
                let o2 = Origin(v2.0, v2.1.clone());
                self.unify(o1, v1.1, o2, v2.1)
            },
            Equation::UnifyMany(mut items) => {
                if !items.is_empty() {
                    // Retrieving first information
                    let EqUnit(l1, mut t1) = items.remove(0);
                    // Unifying with others
                    for EqUnit(l2, t2) in items {
                        t1 = self.unify(
                            Origin(l1.clone(), t1.clone()), t1.clone(),
                            Origin(l2, t2.clone()), t2
                        );
                    }
                    t1
                } else {
                    Typ::Unit
                }
            }
        }
    }

    /// Unifies two types
    pub fn unify(&mut self, o1: Origin, t1: Typ, o2: Origin, t2: Typ) -> Typ {
        // Applying substs
        let t1 = self.apply(t1);
        let t2 = self.apply(t2);
        // Unifying
        if t1 != t2 {
            match (&t1, &t2) {
                (Typ::Prelude(a), Typ::Prelude(b)) => match (a, b) {
                    (PreludeType::Int, PreludeType::Float) => Typ::Prelude(PreludeType::Float),
                    (PreludeType::Float, PreludeType::Int) => Typ::Prelude(PreludeType::Float),
                    _ => bail!(TypeckError::CouldNotUnify {
                        t1: t1.clone(),
                        t2: t2.clone(),
                        related: vec![
                            o1.into_this(),
                            o2.into_with_this(),
                        ]
                    }),
                },
                (Typ::Unbound(a), Typ::Unbound(b)) => {
                    if a == b {
                        t1
                    } else {
                        if self.occurs(*a, &t2) {
                            bail!(TypeckError::TypesRecursion {
                                related: vec![
                                    o1.into_this_type(),
                                    o2.into_with_this(),
                                ],
                                t1: o1.typ(),
                                t2: o2.typ()
                            })
                        }
                        self.hydrator.substitute(*a, t2.clone());
                        t2
                    }
                }
                (Typ::Unbound(a), b) | (b, Typ::Unbound(a)) => {
                    if self.occurs(*a, b) {
                        bail!(TypeckError::TypesRecursion {
                            related: vec![
                                o1.into_this_type(),
                                o2.into_this_type(),
                            ],
                            t1: o1.typ(),
                            t2: o2.typ()
                        })
                    }
                    self.hydrator.substitute(*a, b.clone());
                    b.clone()
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
                    t1
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
                    t1
                }
                (Typ::Function(f1), Typ::Function(f2)) => {
                    f1.params.iter().zip(&f2.params).for_each(|(p1, p2)| {
                        self.unify(o1.clone(), p1.typ.clone(), o2.clone(), p2.typ.clone());
                    });
                    self.unify(o1.clone(), f1.ret.clone(), o2.clone(), f2.ret.clone());
                    t1
                }
                _ => bail!(TypeckError::CouldNotUnify {
                    t1: o1.typ(),
                    t2: o2.typ(),
                    related: vec![
                        o1.into_this_type(),
                        o2.into_this_type(),
                    ]
                }),
            }
        } else {
            t1.clone()
        }
    }

    /// Occurs check
    pub fn occurs(&mut self, own: usize, t: &Typ) -> bool {
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
