/// Imports
use crate::{
    inference::generics::Generics,
    typ::{
        cx::TyCx,
        typ::{GenericArgs, GenericParameter, Typ},
    },
};
use indexmap::IndexMap;
use std::collections::HashMap;

/// A temporary instantiation context used to **hydrate** generic types into
/// fresh inference variables.
///
/// This context performs the *α-renaming* (freshening) of generic parameters
/// when entering an instantiation site — for example, when calling a generic
/// function or constructing a generic struct/enum.
///
/// In practice, `HydrationCx` converts:
///
/// - `Typ::Generic(id)` → a fresh `Typ::Unbound(...)`
///   *unless an explicit substitution is already provided*
///
/// - recursively transforms function types, ADTs (`Struct`, `Enum`) and their
///   generic arguments.
///
/// The context stores two important pieces of data:
///
/// - A reference to the parent [`Hydrator`], used for allocating fresh
///   inference variables.
/// - A local `mapping: HashMap<usize, Typ>` that maps **generic parameter IDs**
///   to the *fresh inference variables* that now stand for them.
///
/// `HydrationCx` is short-lived: it exists only for the duration of a single
/// instantiation (e.g. one function call).
pub struct HydrationCx<'hd, 'tcx> {
    /// Reference to the global hydrator.
    ///
    /// Used to allocate fresh unbound inference variables and to reuse its
    /// substitution machinery.
    hydrator: &'hd mut Hydrator,

    /// Reference to the types contexts.
    ///
    /// Used to retrieve information of `Structs` and `Enums`
    tcx: &'tcx TyCx,

    /// A mapping of **generic parameter IDs** (`Generic(id)`) to the fresh
    /// inference variables created during this instantiation.
    ///
    /// This ensures that generic parameters remain consistent:
    /// `{g(n) -> u(m)}`, reused everywhere within a single instantiation.
    mapping: IndexMap<usize, Typ>,
}

/// Implementation
impl<'hd, 'tcx> HydrationCx<'hd, 'tcx> {
    /// Creates new hydration context
    pub fn new(hydrator: &'hd mut Hydrator, tcx: &'tcx TyCx) -> Self {
        Self {
            hydrator,
            tcx,
            mapping: IndexMap::new(),
        }
    }

    /// Creates new hydration context with given mapping
    pub fn with_mapping(
        hydrator: &'hd mut Hydrator,
        tcx: &'tcx TyCx,
        mapping: IndexMap<usize, Typ>,
    ) -> Self {
        Self {
            hydrator,
            tcx,
            mapping,
        }
    }

    /// Instantiates type by replacing
    /// Generic(id) -> Unbound($id)
    pub fn mk_ty(&mut self, t: Typ) -> Typ {
        match t {
            Typ::Prelude(_) | Typ::Unit => t,
            Typ::Unbound(_) => t,
            Typ::Generic(id) => {
                // If typ is already specified
                if let Some(typ) = self.mapping.get(&id) {
                    typ.clone()
                } else if self.hydrator.is_rigid(id) {
                    Typ::Generic(id)
                } else {
                    let fresh = Typ::Unbound(self.hydrator.fresh());
                    self.mapping.insert(id, fresh.clone());
                    fresh
                }
            }
            Typ::Function(id, args) => {
                let args = args
                    .subtitutions
                    .iter()
                    .map(|(k, v)| (*k, self.mk_ty(v.clone())))
                    .collect();
                let generics = self.mk_generics(&self.tcx.function(id).generics.clone(), args);

                Typ::Function(id, generics)
            }
            Typ::Struct(id, args) => {
                let args = args
                    .subtitutions
                    .iter()
                    .map(|(k, v)| (*k, self.mk_ty(v.clone())))
                    .collect();
                let generics = self.mk_generics(&self.tcx.struct_(id).generics.clone(), args);

                Typ::Struct(id, generics)
            }
            Typ::Enum(id, args) => {
                let args = args
                    .subtitutions
                    .iter()
                    .map(|(k, v)| (*k, self.mk_ty(v.clone())))
                    .collect();
                let generics = self.mk_generics(&self.tcx.enum_(id).generics.clone(), args);

                Typ::Enum(id, generics)
            }
        }
    }

    /// Instantiates generics with args
    /// Generic(id) -> Unbound($id) | Given substitution
    pub fn mk_generics(
        &mut self,
        params: &[GenericParameter],
        args: IndexMap<usize, Typ>,
    ) -> GenericArgs {
        GenericArgs {
            subtitutions: params
                .iter()
                .map(|p| {
                    let generic_id = p.id;
                    (
                        generic_id,
                        self.hydrator
                            .hyd_m(self.tcx, args.clone())
                            .mk_ty(Typ::Generic(generic_id)),
                    )
                })
                .collect(),
        }
    }
}

/// Performs type variable substitution and instantiation during type inference.
///
/// The `Hydrator` is responsible for **resolving unbound type variables**,
/// applying substitutions, and **instantiating generic types** into concrete
/// representations. It operates during the type inference process (unification),
/// ensuring that all types in the type system are fully resolved (i.e., “hydrated”).
///
/// In other words, it "hydrates" abstract type representations by replacing
/// 'Generic' type variables with `Unbound` instances.
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
#[derive(Default, Debug)]
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
            Typ::Enum(id, args) => Typ::Enum(
                id,
                GenericArgs {
                    subtitutions: args
                        .subtitutions
                        .iter()
                        .map(|it| (*it.0, self.apply(it.1.clone())))
                        .collect(),
                },
            ),
            Typ::Struct(id, args) => Typ::Struct(
                id,
                GenericArgs {
                    subtitutions: args
                        .subtitutions
                        .iter()
                        .map(|it| (*it.0, self.apply(it.1.clone())))
                        .collect(),
                },
            ),
            Typ::Function(id, args) => Typ::Function(
                id,
                GenericArgs {
                    subtitutions: args
                        .subtitutions
                        .iter()
                        .map(|it| (*it.0, self.apply(it.1.clone())))
                        .collect(),
                },
            ),
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

    /// Creates hydration context
    pub fn hyd<'a, 'b>(&'a mut self, tcx: &'b TyCx) -> HydrationCx<'a, 'b> {
        HydrationCx::new(self, tcx)
    }

    /// Creates hydration context with given mapping
    pub fn hyd_m<'a, 'b>(
        &'a mut self,
        tcx: &'b TyCx,
        mapping: IndexMap<usize, Typ>,
    ) -> HydrationCx<'a, 'b> {
        HydrationCx::with_mapping(self, tcx, mapping)
    }

    /// Checks that generic is rigid by its ID
    pub fn is_rigid(&self, id: usize) -> bool {
        self.generics.is_rigid(id)
    }
}
