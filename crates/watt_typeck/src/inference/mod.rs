/// Modules
pub mod equation;
pub mod generics;
pub mod hydrator;

/// Imports
use crate::{
    errors::{TypeckError, TypeckRelated},
    inference::{
        equation::{Equation, Unit},
        hydrator::Hydrator,
    },
    typ::typ::{PreludeType, Typ},
};
use log::trace;
use watt_common::{address::Address, bail};

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
            Equation::Unify(v1, v2) => self.unify(v1.0, v1.1, v2.0, v2.1),
            Equation::UnifyMany(items) => {
                if !items.is_empty() {
                    let mut v1: Option<Unit> = None;
                    for v2 in items {
                        v1 = Some(match v1 {
                            Some(v1) => (
                                v1.0.clone() + v2.0.clone(),
                                self.unify(v1.0, v1.1, v2.0, v2.1),
                            ),
                            None => (v2.0, v2.1),
                        });
                    }
                    v1.unwrap().1
                } else {
                    Typ::Unit
                }
            }
        }
    }

    /// Unifies two types
    pub fn unify(&mut self, l1: Address, t1: Typ, l2: Address, t2: Typ) -> Typ {
        // Applying substs
        let t1 = self.apply(t1);
        let t2 = self.apply(t2);
        // Unifying
        println!("unifying: {t1:?} && {t2:?}");
        if t1 != t2 {
            match (&t1, &t2) {
                (Typ::Prelude(a), Typ::Prelude(b)) => match (a, b) {
                    (PreludeType::Int, PreludeType::Float) => Typ::Prelude(PreludeType::Float),
                    (PreludeType::Float, PreludeType::Int) => Typ::Prelude(PreludeType::Float),
                    _ => bail!(TypeckError::CouldNotUnify {
                        t1: t1.clone(),
                        t2: t2.clone(),
                        related: vec![
                            TypeckRelated::This {
                                src: l1.source,
                                span: l1.span.into()
                            },
                            TypeckRelated::WithThis {
                                src: l2.source,
                                span: l2.span.into()
                            }
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
                                    TypeckRelated::ThisType {
                                        src: l1.source,
                                        span: l1.span.into(),
                                        t: t1.clone(),
                                    },
                                    TypeckRelated::ThisType {
                                        src: l2.source,
                                        span: l2.span.into(),
                                        t: t2.clone()
                                    }
                                ],
                                t1: t1.clone(),
                                t2: t2.clone()
                            })
                        }
                        self.hydrator.substitute(*a, t2.clone());
                        t2
                    }
                }
                (Typ::Unbound(a), b) | (b, Typ::Unbound(a)) => {
                    if self.occurs(*a, &b) {
                        bail!(TypeckError::TypesRecursion {
                            related: vec![
                                TypeckRelated::ThisType {
                                    src: l1.source,
                                    span: l1.span.into(),
                                    t: t1.clone(),
                                },
                                TypeckRelated::ThisType {
                                    src: l2.source,
                                    span: l2.span.into(),
                                    t: t2.clone()
                                }
                            ],
                            t1: t1.clone(),
                            t2: t2.clone()
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
                                self.unify(a.location, a.typ, b.location, b.typ);
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
                                        a.location.clone(),
                                        a.typ.clone(),
                                        b.location.clone(),
                                        b.typ.clone(),
                                    );
                                });
                            });
                    }
                    t1
                }
                _ => bail!(TypeckError::CouldNotUnify {
                    t1: t1.clone(),
                    t2: t2.clone(),
                    related: vec![
                        TypeckRelated::ThisType {
                            src: l1.source,
                            span: l1.span.into(),
                            t: t1,
                        },
                        TypeckRelated::ThisType {
                            src: l2.source,
                            span: l2.span.into(),
                            t: t2
                        }
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
