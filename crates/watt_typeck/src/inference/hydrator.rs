/// Imports
use crate::{
    inference::generics::Generics,
    typ::typ::{Function, GenericArgs, GenericParameter, Parameter, Typ},
};
use std::{collections::HashMap, rc::Rc};

/// Performs type variable substitution and instantiation during type inference.
///
/// The `Hydrator` is responsible for **resolving unbound type variables**,
/// applying substitutions, and **instantiating generic types** into concrete
/// representations. It operates during the type inference process (unification),
/// ensuring that all types in the type system are fully resolved (i.e., “hydrated”).
///
/// In other words, it "hydrates" abstract type representations by replacing
/// `Unbound` type variables with actual, concrete `Typ` instances.
///
/// # Fields
///
/// - `substitutions: HashMap<usize, Typ>`
///   A mapping of **unbound type variable IDs** to their corresponding resolved types.
///   During unification, whenever a variable is assigned a type, that binding is stored here.
///   The hydrator uses this map to recursively replace variable references with their final types
///   during the type instantiation.
///
/// - `last_unbound_id: usize`
///   The **last generated unbound type ID**, used to generate fresh uid
///   for new unbound type variables during inference.
///   This acts like a counter for generating unique type variable identifiers.
///
/// - `generics: Generics`
///   Tracks **generic parameters** visible in the current scope.
///   This allows the hydrator to distinguish between *generic* and *inference* variables,
///   and to correctly instantiate generics when entering or leaving scopes.
///
/// # Typical Responsibilities
///
/// 1. **Apply substitutions**
///    Recursively replaces all unbound type variables (`Typ::Unbound(id)`) with their
///    corresponding resolved types from the `substitutions` map.
///
/// 2. **Instantiate generics**
///    When a generic type is used, the hydrator creates a fresh unbound type variable
///    for each generic parameter (α-renaming).
///
/// 3. **Create and track unbound variables**
///    Generates fresh type variables during inference when type information
///    is not yet available.
///
/// 4. **Ensure type consistency**
///    Guarantees that all types in the final AST are concrete and that no unbound
///    or partially inferred types remain.
///
/// # Example
///
/// ```rust
/// let mut hydrator = Hydrator::default();
///
/// // Suppose type variable 1 was unified to `Int`
/// hydrator.substitute(1, Typ::Int);
///
/// // Later, a type like `List<Unbound(1)>` can be "hydrated" into `List<Int>`
/// let hydrated = hydrator.apply(Typ::Unbound(1));
/// assert_eq!(hydrated, Typ::Int);
/// ```
///
#[derive(Default)]
pub struct Hydrator {
    /// Mapping of unbound type variable IDs to resolved types.
    substitutions: HashMap<usize, Typ>,

    /// The last generated unbound type ID.
    last_unbound_id: usize,

    /// The currently active generic scopes.
    pub(crate) generics: Generics,
}

/// Implementation
impl Hydrator {
    /// Creates a substitution pair in substitutions map
    ///
    /// # Parameters
    /// - `id: usize`
    ///   Unbound id, with what we need to creates substitution
    ///
    /// - `typ: Typ`
    ///   The type that we using to create substitution
    ///
    /// # Notes
    /// If substitution is already exists, this function
    /// isn't updating the already created substitution.
    ///
    pub fn substitute(&mut self, id: usize, typ: Typ) {
        self.substitutions.entry(id).or_insert(typ);
    }

    /// Applies a substitutions for the given typ
    ///
    /// # Parameters
    /// - `typ: Typ`
    ///   The type that we using to apply substitution
    ///
    pub fn apply(&self, typ: Typ) -> Typ {
        match typ {
            it @ Typ::Unbound(usize) => self
                .substitutions
                .get(&usize)
                .map_or(it, |s| self.apply(s.clone())),
            Typ::Enum(en, args) => Typ::Enum(
                en,
                GenericArgs {
                    subtitutions: args
                        .subtitutions
                        .iter()
                        .map(|it| (*it.0, self.apply(it.1.clone())))
                        .collect(),
                },
            ),
            Typ::Struct(ty, args) => Typ::Struct(
                ty,
                GenericArgs {
                    subtitutions: args
                        .subtitutions
                        .iter()
                        .map(|it| (*it.0, self.apply(it.1.clone())))
                        .collect(),
                },
            ),
            Typ::Function(f) => Typ::Function(Rc::new(Function {
                location: f.location.clone(),
                name: f.name.clone(),
                generics: f.generics.clone(),
                params: f.params.iter().cloned().map(|p| Parameter {
                    location: p.location,
                    typ: self.apply(p.typ),
                }).collect(),
                ret: self.apply(f.ret.clone()),
            })),
            other => other,
        }
    }

    /// Generates fresh unique id
    /// for the unbound type variable.
    ///
    pub fn fresh(&mut self) -> usize {
        self.last_unbound_id += 1;
        self.last_unbound_id
    }

    /// Generates fresh unique id
    /// for the unbound type variable.
    ///
    /// Then creates substitution, returns
    /// unbound type variable unique id
    ///
    pub fn bind(&mut self, to: Typ) -> usize {
        let id = self.fresh();
        self.substitute(id, to);
        id
    }

    /// Instantiates type by replacing
    /// Generic(id) -> Unbound($id)
    pub fn instantiate(&mut self, t: Typ, generics: &mut HashMap<usize, Typ>) -> Typ {
        match t {
            Typ::Prelude(_) | Typ::Unit => t,
            Typ::Unbound(_) => t,
            Typ::Generic(id) => {
                // If typ is already specified
                if let Some(typ) = generics.get(&id) {
                    typ.clone()
                } else {
                    let fresh = Typ::Unbound(self.fresh());
                    generics.insert(id, fresh.clone());
                    fresh
                }
            }
            Typ::Function(rc) => Typ::Function(self.mk_function(rc, generics)),
            Typ::Struct(rc, args) => {
                let mut args = args
                    .subtitutions
                    .iter()
                    .map(|(k, v)| (*k, self.instantiate(v.clone(), generics)))
                    .collect();
                let generics = self.mk_generics(&rc.borrow().generics, &mut args);

                Typ::Struct(rc, generics)
            }
            Typ::Enum(rc, args) => {
                let mut args = args
                    .subtitutions
                    .iter()
                    .map(|(k, v)| (*k, self.instantiate(v.clone(), generics)))
                    .collect();
                let generics = self.mk_generics(&rc.borrow().generics, &mut args);

                Typ::Enum(rc, generics)
            }
        }
    }

    /// Instantiates generics with args
    /// Generic(id) -> Unbound($id)
    pub fn mk_generics(
        &mut self,
        params: &[GenericParameter],
        args: &mut HashMap<usize, Typ>,
    ) -> GenericArgs {
        GenericArgs {
            subtitutions: params
                .iter()
                .map(|p| {
                    let generic_id = p.id;
                    (generic_id, self.instantiate(Typ::Generic(generic_id), args))
                })
                .collect(),
        }
    }

    /// Instantiates function by replacing
    /// Generic(id) -> Unbound($id)
    pub fn mk_function(
        &mut self,
        rc: Rc<Function>,
        generics: &mut HashMap<usize, Typ>,
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
}
