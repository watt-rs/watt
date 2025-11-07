/// Imports
use crate::{
    cx::{module::ModuleCx, package::PackageCx},
    errors::{TypeckError, TypeckRelated},
    resolve::resolve::ModuleResolver,
    typ::{CustomType, Enum, EnumVariant, Function, Parameter, PreludeType, Struct, Typ},
};
use ecow::EcoString;
use log::trace;
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use watt_ast::ast::TypePath;
use watt_common::{address::Address, bail};

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
    #[allow(unused)]
    package: &'cx PackageCx<'cx>,
    /// Type variable last id
    last_typ_var_id: usize,
    /// Type variables subtitutions
    substitutions: HashMap<usize, Typ>,
}

/// Implementation
impl<'cx> EquationsSolver<'cx> {
    /// Creates new equations solver
    pub fn new(package: &'cx PackageCx<'cx>) -> Self {
        Self {
            package,
            last_typ_var_id: 0,
            substitutions: HashMap::new(),
        }
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

    /// Gives type from ast
    pub fn type_from_ast(
        &mut self,
        path: TypePath,
        map: HashMap<EcoString, TypePath>,
        resolver: &ModuleResolver,
    ) -> Typ {
        match path {
            TypePath::Local {
                location,
                name,
                generics,
            } => match name.as_str() {
                "int" if generics.len() == 0 => Typ::Prelude(PreludeType::Int),
                "float" if generics.len() == 0 => Typ::Prelude(PreludeType::Float),
                "bool" if generics.len() == 0 => Typ::Prelude(PreludeType::Bool),
                "string" if generics.len() == 0 => Typ::Prelude(PreludeType::String),
                "dyn" if generics.len() == 0 => Typ::Dyn,
                "unit" if generics.len() == 0 => Typ::Unit,
                _ => match map.get(&name) {
                    Some(g) => self.type_from_ast(g.clone(), map, resolver),
                    None => match resolver.resolve_type(&name, &location) {
                        CustomType::Enum(en) => Typ::Enum(if generics.len() != en.generics.len() {
                            bail!(TypeckError::ArityMissmatch {
                                related: vec![TypeckRelated::This {
                                    src: en.location.source.clone(),
                                    span: location.span.into()
                                }],
                                expected: en.generics.len(),
                                got: generics.len()
                            })
                        } else {
                            self.type_from_ast(en.clone(), map, resolver)
                        }),
                        CustomType::Struct(ty) => {
                            Typ::Struct(self.type_from_ast(ty.clone(), map, resolver))
                        }
                        CustomType::Trait(tr) => Typ::Trait(tr.clone()),
                    },
                },
            },
            TypePath::Module {
                location,
                module,
                name,
                generics,
            } => {
                let m = self.resolver.resolve_module(&module);

                match m.fields.get(&name) {
                    Some(field) => match field {
                        ModDef::CustomType(t) => {
                            if t.publicity != Publicity::Private {
                                match &t.value {
                                    CustomType::Enum(en) => Typ::Enum(en.clone()),
                                    CustomType::Struct(ty) => Typ::Struct(ty.clone()),
                                    CustomType::Trait(tr) => Typ::Trait(tr.clone()),
                                }
                            } else {
                                bail!(TypeckError::TypeIsPrivate {
                                    src: self.module.source.clone(),
                                    span: location.span.into(),
                                    t: t.value.clone()
                                })
                            }
                        }
                        ModDef::Variable(_) => bail!(TypeckError::CouldNotUseValueAsType {
                            src: self.module.source.clone(),
                            span: location.clone().span.into(),
                            v: name
                        }),
                    },
                    None => bail!(TypeckError::TypeIsNotDefined {
                        src: self.module.source.clone(),
                        span: location.span.into(),
                        t: format!("{module}.{name}").into()
                    }),
                }
            }
            TypePath::Function {
                location,
                generics,
                params,
                ret,
            } => {
                self.generics.enter(generics);
                let function = Typ::Function(Rc::new(Function {
                    source: self.module.source.clone(),
                    location,
                    uid: self.fresh_id(),
                    name: EcoString::from("$annotated"),
                    params: params
                        .into_iter()
                        .map(|p| Parameter {
                            location: p.location(),
                            typ: self.infer_type_annotation(p),
                        })
                        .collect(),
                    ret: ret.map_or(Typ::Unit, |t| self.infer_type_annotation(*t)),
                }));
                self.generics.exit();
                function
            }
            TypePath::Unit { .. } => Typ::Unit,
        }
    }

    /// Instantiates type by replacing
    /// Generic(id) -> Unbound($id)
    pub fn instantiate(&mut self, t: Typ, generics: &mut HashMap<EcoString, usize>) -> Typ {
        match t {
            Typ::Prelude(_) | Typ::Unit | Typ::Dyn => t,
            Typ::Unbound(_) => t,
            Typ::Generic(id) => {
                // If unbound is already generated
                if let Some(&unbound_id) = generics.get(&id) {
                    Typ::Unbound(unbound_id)
                } else {
                    let fresh = self.fresh();
                    generics.insert(id, fresh);
                    Typ::Unbound(fresh)
                }
            }
            Typ::Function(rc) => Typ::Function(self.instantiate_function(rc, generics)),
            Typ::Struct(rc) => Typ::Struct(self.instantiate_struct(rc, generics)),
            Typ::Enum(rc) => Typ::Enum(self.instantiate_enum(rc, generics)),
            Typ::Trait(rc) => Typ::Trait(rc.clone()),
        }
    }

    /// Instantiates struct by replacing
    /// Generic(id) -> Unbound($id)
    pub fn instantiate_struct(
        &mut self,
        rc: Rc<RefCell<Struct>>,
        generics: &mut HashMap<EcoString, usize>,
    ) -> Rc<RefCell<Struct>> {
        let strct = rc.borrow();
        let params = strct
            .params
            .iter()
            .cloned()
            .map(|p| Parameter {
                location: p.location,
                typ: self.instantiate(p.typ, generics),
            })
            .collect();

        Rc::new(RefCell::new(Struct {
            source: strct.source.clone(),
            location: strct.location.clone(),
            uid: strct.uid,
            generics: strct.generics.clone(),
            name: strct.name.clone(),
            params,
            env: strct.env.clone(),
        }))
    }

    /// Instantiates enum by replacing
    /// Generic(id) -> Unbound($id)
    pub fn instantiate_enum(
        &mut self,
        rc: Rc<Enum>,
        generics: &mut HashMap<EcoString, usize>,
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
            source: rc.source.clone(),
            location: rc.location.clone(),
            uid: rc.uid,
            generics: rc.generics.clone(),
            name: rc.name.clone(),
            variants,
        })
    }

    /// Instantiates function by replacing
    /// Generic(id) -> Unbound($id)
    pub fn instantiate_function(
        &mut self,
        rc: Rc<Function>,
        generics: &mut HashMap<EcoString, usize>,
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
            source: rc.source.clone(),
            location: rc.location.clone(),
            uid: rc.uid,
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
                        self.substitutions.insert(*a, t2.clone());
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
                    self.substitutions.insert(*a, t2.clone());
                    t2
                }
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
            Typ::Struct(ref rc) => rc.borrow().params.iter().any(|p| self.occurs(own, &p.typ)),
            Typ::Enum(ref en) => en
                .variants
                .iter()
                .any(|v| v.params.iter().any(|p| self.occurs(own, &p.1))),
            Typ::Trait(_) | Typ::Prelude(_) | Typ::Unit | Typ::Dyn => false,
            Typ::Generic(_) => unreachable!(),
        }
    }

    /// Applies substitutions to type
    pub fn apply(&mut self, typ: Typ) -> Typ {
        match typ {
            it @ Typ::Unbound(usize) => self.substitutions.get(&usize).map_or(it, |s| s.clone()),
            other => other,
        }
    }

    /// Gives fresh unbound id
    pub fn fresh(&mut self) -> usize {
        self.last_typ_var_id += 1;
        self.last_typ_var_id
    }
}
