/// Imports
use crate::{
    errors::TypeckError,
    typ::{PreludeType, Typ},
};
use miette::NamedSource;
use std::sync::Arc;
use watt_common::{address::Address, bail};

/// Equation var
pub type Var = (Address, Typ);

/// Equation
pub enum Equation {
    Unify(Var, Var),
    UnifyMany(Vec<Var>),
}

/// Equations solver
pub struct EquationsSolver<'eq> {
    source: &'eq NamedSource<Arc<String>>,
}

/// Implementation
impl<'eq> EquationsSolver<'eq> {
    /// Creates new equations solver
    pub fn new(source: &'eq NamedSource<Arc<String>>) -> Self {
        Self { source }
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
                (Typ::Dyn, _) | (_, Typ::Dyn) => Typ::Dyn,
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
