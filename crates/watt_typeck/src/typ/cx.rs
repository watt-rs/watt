/// Imports
use crate::{
    inference::generics::Generics,
    typ::typ::{Enum, Function, GenericArgs, GenericParameter, Struct, TyVar, Typ},
};
use id_arena::{Arena, Id};
use indexmap::IndexMap;

/// Type context.
///
/// `TyCx` owns and stores all type-level definitions used by the compiler,
/// such as functions, structs, and enums.
///
/// All definitions are allocated in arenas and are referenced indirectly
/// via typed IDs (`Id<T>`). This provides:
///
/// - zero-cost copying of references
/// - stable identities for types
/// - support for recursive and cyclic type graphs
/// - clear separation between type *references* and type *definitions*
///
/// `TyCx` is expected to live for the entire duration of type checking
/// and later compilation phases.
///
#[derive(Default)]
pub struct TyCx {
    /// Arena storing all function type definitions.
    pub funcs: Arena<Function>,

    /// Arena storing all struct definitions.
    pub structs: Arena<Struct>,

    /// Arena storing all enum definitions.
    pub enums: Arena<Enum>,
}

impl TyCx {
    /// Allocates a new function definition in the type context
    /// and returns its unique ID.
    #[inline]
    pub fn insert_function(&mut self, function: Function) -> Id<Function> {
        self.funcs.alloc(function)
    }

    /// Allocates a new struct definition in the type context
    /// and returns its unique ID.
    #[inline]
    pub fn insert_struct(&mut self, struct_: Struct) -> Id<Struct> {
        self.structs.alloc(struct_)
    }

    /// Allocates a new enum definition in the type context
    /// and returns its unique ID.
    #[inline]
    pub fn insert_enum(&mut self, enum_: Enum) -> Id<Enum> {
        self.enums.alloc(enum_)
    }

    /// Returns an immutable reference to a function definition.
    ///
    /// # Panics
    ///
    /// Panics if the given `id` does not belong to this `TyCx`.
    #[inline]
    pub fn function(&self, id: Id<Function>) -> &Function {
        self.funcs.get(id).expect("invalid Function id")
    }

    /// Returns an immutable reference to a struct definition.
    ///
    /// # Panics
    ///
    /// Panics if the given `id` does not belong to this `TyCx`.
    #[inline]
    pub fn struct_(&self, id: Id<Struct>) -> &Struct {
        self.structs.get(id).expect("invalid Struct id")
    }

    /// Returns an immutable reference to an enum definition.
    ///
    /// # Panics
    ///
    /// Panics if the given `id` does not belong to this `TyCx`.
    #[inline]
    pub fn enum_(&self, id: Id<Enum>) -> &Enum {
        self.enums.get(id).expect("invalid Enum id")
    }

    /// Returns a mutable reference to a function definition.
    ///
    /// # Panics
    ///
    /// Panics if the given `id` does not belong to this `TyCx`.
    #[inline]
    pub fn function_mut(&mut self, id: Id<Function>) -> &mut Function {
        self.funcs.get_mut(id).expect("invalid Function id")
    }

    /// Returns a mutable reference to a struct definition.
    ///
    /// # Panics
    ///
    /// Panics if the given `id` does not belong to this `TyCx`.
    #[inline]
    pub fn struct_mut(&mut self, id: Id<Struct>) -> &mut Struct {
        self.structs.get_mut(id).expect("invalid Struct id")
    }

    /// Returns a mutable reference to an enum definition.
    ///
    /// # Panics
    ///
    /// Panics if the given `id` does not belong to this `TyCx`.
    #[inline]
    pub fn enum_mut(&mut self, id: Id<Enum>) -> &mut Enum {
        self.enums.get_mut(id).expect("invalid Enum id")
    }

    /// Executes `f` with an immutable reference to the function definition
    /// if the given `id` exists in this context.
    ///
    /// Returns `Some(result)` if the function was found, or `None` otherwise.
    #[inline]
    pub fn with_function<R>(&self, id: Id<Function>, f: impl FnOnce(&Function) -> R) -> Option<R> {
        self.funcs.get(id).map(f)
    }

    /// Executes `f` with an immutable reference to the struct definition
    /// if the given `id` exists in this context.
    #[inline]
    pub fn with_struct<R>(&self, id: Id<Struct>, f: impl FnOnce(&Struct) -> R) -> Option<R> {
        self.structs.get(id).map(f)
    }

    /// Executes `f` with an immutable reference to the enum definition
    /// if the given `id` exists in this context.
    #[inline]
    pub fn with_enum<R>(&self, id: Id<Enum>, f: impl FnOnce(&Enum) -> R) -> Option<R> {
        self.enums.get(id).map(f)
    }
}

/// Performs type variable substitution and instantiation during type inference.
///
/// The `InferCx` is responsible for **resolving unbound type variables**,
/// applying substitutions, and **instantiating generic types** into concrete
/// representations. It operates during the type inference process (unification),
/// ensuring that all types in the type system are fully resolved (i.e., “inferred”).
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
pub struct InferCx<'tcx> {
    /// Represents types context
    pub(crate) tcx: &'tcx mut TyCx,

    /// Mapping of type variable IDs to resolved types.
    type_variables: Arena<TyVar>,

    /// The currently active generic scopes.
    pub(crate) generics: Generics,
}

/// Implementation
impl<'tcx> InferCx<'tcx> {
    /// Creates new inference context
    ///
    /// # Parameters
    /// - `tcx: &'tcx mut TyCx`
    ///   Types context
    ///
    pub fn new(tcx: &'tcx mut TyCx) -> Self {
        Self {
            tcx,
            type_variables: Arena::new(),
            generics: Generics::default(),
        }
    }

    /// Creates a substitution pair in substitutions map
    ///
    /// # Parameters
    /// - `id: Id<TyVar>`
    ///   Type variable id, with what we need to creates substitution
    /// - `typ: Typ`
    ///   The type that we using to create substitution
    ///
    /// # Notes
    /// If substitution is already exists, this function
    /// isn't updating the already created substitution.
    ///
    pub fn substitute(&mut self, id: Id<TyVar>, typ: Typ) {
        let var = self.type_variables.get_mut(id).expect("invalid TyVar id");
        if let TyVar::Unbound = var {
            *var = TyVar::Bound(typ);
        }
    }

    /// Generates fresh unbound type variable.
    ///
    pub fn fresh(&mut self) -> Id<TyVar> {
        self.type_variables.alloc(TyVar::Unbound)
    }

    /// Instantiates type by replacing
    ///
    /// `Typ::Generic(id)` → a fresh `Typ::Unbound(...)`
    ///
    pub fn mk_fresh(&mut self, typ: Typ) -> Typ {
        FresheningCx::fresh(self, typ)
    }

    /// Instantiates type by replacing
    ///
    /// `Typ::Generic(id)` → a fresh `Typ::Unbound(...)`
    ///   *unless an explicit substitution is already provided*
    ///
    pub fn mk_fresh_m(&mut self, typ: Typ, m: IndexMap<usize, Typ>) -> Typ {
        FresheningCx::fresh_m(self, typ, m)
    }

    /// Retrieves generic args by replacing
    ///
    /// `Typ::Generic(id)` → a fresh `Typ::Unbound(...)`
    ///
    pub fn mk_fresh_generics<'a>(&'a mut self, generics: &[GenericParameter]) -> GenericArgs {
        FresheningCx::new(self).mk_generics(generics, IndexMap::new())
    }

    /// Retrieves generic args by replacing
    ///
    /// `Typ::Generic(id)` → a fresh `Typ::Unbound(...)`
    ///
    pub fn mk_fresh_generics_m<'a>(
        &'a mut self,
        generics: &[GenericParameter],
        m: IndexMap<usize, Typ>,
    ) -> GenericArgs {
        FresheningCx::new(self).mk_generics(generics, m)
    }

    /// Generates fresh type variable bound to given type.
    ///
    pub fn bind(&mut self, to: Typ) -> Id<TyVar> {
        self.type_variables.alloc(TyVar::Bound(to))
    }

    /// Return immutable reference to the type variable by id
    ///
    pub fn get(&self, id: Id<TyVar>) -> &TyVar {
        self.type_variables.get(id).expect("invalid TyVar id")
    }

    /// Return mutable reference to the type variable by id
    ///
    pub fn get_mut(&mut self, id: Id<TyVar>) -> &mut TyVar {
        self.type_variables.get_mut(id).expect("invalid TyVar id")
    }

    /// Applies a substitutions for the given typ
    ///
    /// # Parameters
    /// - `typ: Typ`
    ///   The type that we using to apply substitution
    ///
    pub fn apply(&self, typ: Typ) -> Typ {
        match typ {
            Typ::Var(id) => match self.get(id) {
                TyVar::Unbound => typ,
                TyVar::Bound(typ) => typ.clone(),
            },
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

    /// Checks that generic is rigid by its ID
    ///
    pub fn is_rigid(&self, id: usize) -> bool {
        self.generics.is_rigid(id)
    }
}

/// A temporary instantiation context used to replace generic types with
/// fresh inference variables.
///
/// This context performs the *α-renaming* (freshening) of generic parameters
/// when entering an instantiation site — for example, when calling a generic
/// function or constructing a generic struct/enum.
///
/// In practice, `FresheningCx` converts:
///
/// - `Typ::Generic(id)` → a fresh `Typ::Unbound(...)`
///   *unless an explicit substitution is already provided*
///
/// - recursively transforms function types, ADTs (`Struct`, `Enum`) and their
///   generic arguments.
///
/// The context stores two important pieces of data:
///
/// - A reference to the `InferCx`, used for allocating fresh
///   inference variables.
/// - A local `mapping: HashMap<usize, Typ>` that maps **generic parameter IDs**
///   to the *fresh inference variables* that now stand for them.
///
/// `FresheningCx` is short-lived: it exists only for the duration of a single
/// instantiation (e.g. one function call).
pub struct FresheningCx<'icx, 'tcx> {
    /// Reference to the inference cx
    icx: &'icx mut InferCx<'tcx>,

    /// A mapping of **generic parameter IDs** (`Generic(id)`) to the fresh
    /// inference variables created during this instantiation.
    ///
    /// This ensures that generic parameters remain consistent:
    /// `{g(n) -> u(m)}`, reused everywhere within a single instantiation.
    mapping: IndexMap<usize, Typ>,
}

/// Implementation
impl<'icx, 'tcx> FresheningCx<'icx, 'tcx> {
    /// Creates new freshening context
    pub fn new(icx: &'icx mut InferCx<'tcx>) -> Self {
        Self {
            icx,
            mapping: IndexMap::new(),
        }
    }

    /// Performs freshening of the type
    pub fn fresh(icx: &'icx mut InferCx<'tcx>, typ: Typ) -> Typ {
        let mut fcx = Self {
            icx,
            mapping: IndexMap::new(),
        };
        fcx.mk_ty(typ)
    }

    /// Creates new hydration context with given mapping
    pub fn fresh_m(icx: &'icx mut InferCx<'tcx>, typ: Typ, mapping: IndexMap<usize, Typ>) -> Typ {
        let mut fcx = Self { icx, mapping };
        fcx.mk_ty(typ)
    }

    /// Instantiates type by replacing
    /// Generic(id) -> Unbound($id)
    pub fn mk_ty(&mut self, t: Typ) -> Typ {
        match t {
            Typ::Prelude(_) | Typ::Unit | Typ::Var(_) => t,
            Typ::Generic(id) => {
                // If typ is already specified
                if let Some(typ) = self.mapping.get(&id) {
                    typ.clone()
                } else if self.icx.is_rigid(id) {
                    Typ::Generic(id)
                } else {
                    let fresh = Typ::Var(self.icx.fresh());
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
                let generics = self.mk_generics(&self.icx.tcx.function(id).generics.clone(), args);

                Typ::Function(id, generics)
            }
            Typ::Struct(id, args) => {
                let args = args
                    .subtitutions
                    .iter()
                    .map(|(k, v)| (*k, self.mk_ty(v.clone())))
                    .collect();
                let generics = self.mk_generics(&self.icx.tcx.struct_(id).generics.clone(), args);

                Typ::Struct(id, generics)
            }
            Typ::Enum(id, args) => {
                let args = args
                    .subtitutions
                    .iter()
                    .map(|(k, v)| (*k, self.mk_ty(v.clone())))
                    .collect();
                let generics = self.mk_generics(&self.icx.tcx.enum_(id).generics.clone(), args);

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
                    match args.get(&generic_id) {
                        Some(s) => (generic_id, s.clone()),
                        None => (generic_id, Typ::Var(self.icx.fresh())),
                    }
                })
                .collect(),
        }
    }
}
