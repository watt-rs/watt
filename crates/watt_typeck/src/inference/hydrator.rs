/// Imports
use crate::{inference::generics::Generics, typ::typ::Typ};
use std::collections::HashMap;

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
    generics: Generics,
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
        if !self.substitutions.contains_key(&id) {
            self.substitutions.insert(id, typ);
        }
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
}
