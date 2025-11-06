/// Imports
use crate::{
    cx::package::PackageCx,
    errors::{TypeckError, TypeckRelated},
    typ::{PreludeType, Typ},
    warnings::TypeckWarning,
};
use log::trace;
use watt_common::{address::Address, bail, warn};

/// Equation var
pub type Var = (Address, Typ);

/// Equation
#[derive(Debug, Clone)]
pub enum Equation {
    Unify(Var, Var),
    UnifyMany(Vec<Var>),
}

/// Equations solver
pub struct EquationsSolver<'cx> {
    /// Package context
    package: &'cx PackageCx<'cx>,
}

/// Implementation
impl<'cx> EquationsSolver<'cx> {
    /// Creates new equations solver
    pub fn new(package: &'cx PackageCx<'cx>) -> Self {
        Self { package }
    }

    /// Solves the equation
    pub fn solve(&mut self, equation: Equation) -> Typ {
        trace!("solving equation: {equation:?}");
        // Solving
        match equation.clone() {
            Equation::Unify(v1, v2) => self.unify(v1.0, v1.1, v2.0, v2.1),
            Equation::UnifyMany(items) => {
                if !items.is_empty() {
                    let mut v1: Option<Var> = None;
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
        trace!("unifying: {t1:?} && {t2:?}");
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
                (Typ::Dyn, t) | (t, Typ::Dyn) => match t {
                    Typ::Unit => {
                        warn!(
                            self.package,
                            TypeckWarning::UnitAndDynUnification {
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
                            }
                        );
                        Typ::Dyn
                    }
                    _ => Typ::Dyn,
                },
                (Typ::Trait(tr), Typ::Struct(ty)) | (Typ::Struct(ty), Typ::Trait(tr)) => {
                    if ty.borrow().is_impls(tr.clone()) {
                        Typ::Trait(tr.clone())
                    } else {
                        bail!(TypeckError::CouldNotUnifyTraitAndTyp {
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
                            tr: t1.clone(),
                            ty: t2.clone()
                        })
                    }
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
}
