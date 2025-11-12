/// Modules
pub mod equation;
pub mod generics;
pub mod hydrator;

use std::{collections::HashMap, rc::Rc};

/// Imports
use crate::{
    cx::package::PackageCx,
    errors::{TypeckError, TypeckRelated},
    inference::{
        equation::{Equation, Unit},
        hydrator::Hydrator,
    },
    typ::typ::{Enum, EnumVariant, Field, Function, Parameter, PreludeType, Struct, Typ},
};
use log::trace;
use watt_common::{address::Address, bail};

/// Equations solver
pub struct EquationsSolver<'cx> {
    /// Package context
    package: &'cx PackageCx<'cx>,
    /// Module hydrator
    hydrator: Hydrator,
}

/// Implementation
impl<'cx> EquationsSolver<'cx> {
    /// Creates new equations solver
    pub fn new(package: &'cx PackageCx<'cx>, hydrator: Hydrator) -> Self {
        Self { package, hydrator }
    }

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

    /// Instantiates type by replacing
    /// Generic(id) -> Unbound($id)
    pub fn instantiate(&mut self, t: Typ, generics: &mut HashMap<usize, usize>) -> Typ {
        match t {
            Typ::Prelude(_) | Typ::Unit => t,
            Typ::Unbound(_) => t,
            Typ::Generic(id) => {
                // If unbound is already generated
                if let Some(&unbound_id) = generics.get(&id) {
                    Typ::Unbound(unbound_id)
                } else {
                    let fresh = self.hydrator.fresh();
                    generics.insert(id, fresh);
                    Typ::Unbound(fresh)
                }
            }
            Typ::Function(rc) => Typ::Function(self.instantiate_function(rc, generics)),
            Typ::Struct(rc) => Typ::Struct(self.instantiate_struct(rc, generics)),
            Typ::Enum(rc) => Typ::Enum(self.instantiate_enum(rc, generics)),
        }
    }

    /// Instantiates struct by replacing
    /// Generic(id) -> Unbound($id)
    pub fn instantiate_struct(
        &mut self,
        rc: Rc<Struct>,
        generics: &mut HashMap<usize, usize>,
    ) -> Rc<Struct> {
        Rc::new(Struct {
            location: rc.location.clone(),
            uid: rc.uid,
            name: rc.name.clone(),
            generics: rc.generics.clone(),
            fields: rc
                .fields
                .iter()
                .map(|f| Field {
                    name: f.name.clone(),
                    location: f.location.clone(),
                    typ: self.instantiate(f.typ.clone(), generics),
                })
                .collect(),
        })
    }

    /// Instantiates enum by replacing
    /// Generic(id) -> Unbound($id)
    pub fn instantiate_enum(
        &mut self,
        rc: Rc<Enum>,
        generics: &mut HashMap<usize, usize>,
    ) -> Rc<Enum> {
        let variants = rc
            .variants
            .iter()
            .cloned()
            .map(|v| EnumVariant {
                location: v.location,
                name: v.name,
                params: v
                    .params
                    .into_iter()
                    .map(|p| (p.0, self.instantiate(p.1, generics)))
                    .collect(),
            })
            .collect();

        Rc::new(Enum {
            location: rc.location.clone(),
            uid: rc.uid,
            name: rc.name.clone(),
            generics: rc.generics.clone(),
            variants,
        })
    }

    /// Instantiates function by replacing
    /// Generic(id) -> Unbound($id)
    pub fn instantiate_function(
        &mut self,
        rc: Rc<Function>,
        generics: &mut HashMap<usize, usize>,
    ) -> Rc<Function> {
        let params = rc
            .params
            .iter()
            .cloned()
            .map(|p| Parameter {
                location: p.location,
                typ: self.instantiate(p.typ, generics),
            })
            .collect();

        let ret = self.instantiate(rc.ret.clone(), generics);
        Rc::new(Function {
            location: rc.location.clone(),
            name: rc.name.clone(),
            generics: rc.generics.clone(),
            params,
            ret,
        })
    }

    /// Unifies two types
    pub fn unify(&mut self, l1: Address, t1: Typ, l2: Address, t2: Typ) -> Typ {
        // Applying substs
        let t1 = self.apply(t1);
        let t2 = self.apply(t2);
        // Unifying
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
                (Typ::Unbound(a), Typ::Unbound(b)) => {
                    if a == b {
                        t1
                    } else {
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
                    self.hydrator.substitute(*a, t2.clone());
                    t2
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
            Typ::Struct(ref rc) => rc.fields.iter().any(|f| self.occurs(own, &f.typ)),
            Typ::Enum(ref en) => en
                .variants
                .iter()
                .any(|v| v.params.iter().any(|p| self.occurs(own, &p.1))),
            Typ::Generic(_) | Typ::Prelude(_) | Typ::Unit => false,
        }
    }

    /// Applies substitutions to type
    pub fn apply(&mut self, typ: Typ) -> Typ {
        self.hydrator.apply(typ)
    }
}
