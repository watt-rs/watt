/// Imports
use crate::{
    cx::package::PackageCx,
    errors::TypeckError,
    typ::{PreludeType, Typ},
    warnings::TypeckWarning,
};
use miette::NamedSource;
use std::sync::Arc;
use watt_common::{address::Address, bail, warn};

/// Equation var
pub type Var = (Address, Typ);

/// Equation
pub enum Equation {
    Unify(Var, Var),
    UnifyMany(Vec<Var>),
}

/// Equations solver
pub struct EquationsSolver<'eq, 'cx> {
    source: &'eq NamedSource<Arc<String>>,
    package: &'cx PackageCx<'cx>,
}

/// Implementation
impl<'eq, 'cx> EquationsSolver<'eq, 'cx> {
    /// Creates new equations solver
    pub fn new(package: &'cx PackageCx<'cx>, source: &'eq NamedSource<Arc<String>>) -> Self {
        Self { package, source }
    }

    /// Solves the equation
    pub fn solve(&mut self, equation: Equation) -> Typ {
        match equation {
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
        if t1 != t2 {
            match (&t1, &t2) {
                (Typ::Prelude(a), Typ::Prelude(b)) => match (a, b) {
                    (PreludeType::Int, PreludeType::Float) => Typ::Prelude(PreludeType::Float),
                    (PreludeType::Float, PreludeType::Int) => Typ::Prelude(PreludeType::Float),
                    _ => bail!(TypeckError::CouldNotUnify {
                        src: self.source.clone(),
                        first_span: l1.span.clone().into(),
                        t1: t1.clone(),
                        second_span: l2.span.clone().into(),
                        t2: t2.clone()
                    }),
                },
                (Typ::Dyn, t) | (t, Typ::Dyn) => match t {
                    Typ::Unit => {
                        warn!(
                            self.package,
                            TypeckWarning::UnitAndDynUnification {
                                src: self.source.clone(),
                                first_span: l1.span.clone().into(),
                                second_span: l2.span.clone().into(),
                            }
                        );
                        Typ::Dyn
                    }
                    _ => Typ::Dyn,
                },
                (Typ::Trait(tr), Typ::Custom(ty)) | (Typ::Custom(ty), Typ::Trait(tr)) => {
                    if ty.borrow().is_impls(tr.clone()) {
                        Typ::Trait(tr.clone())
                    } else {
                        bail!(TypeckError::CouldNotUnifyTraitAndTyp {
                            src: self.source.clone(),
                            first_span: l1.span.clone().into(),
                            tr: t1.clone(),
                            second_span: l2.span.clone().into(),
                            ty: t2.clone()
                        })
                    }
                }
                _ => bail!(TypeckError::CouldNotUnify {
                    src: self.source.clone(),
                    first_span: l1.span.clone().into(),
                    t1: t1.clone(),
                    second_span: l2.span.clone().into(),
                    t2: t2.clone()
                }),
            }
        } else {
            t1.clone()
        }
    }
}
