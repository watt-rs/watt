/// Imports
use crate::typ::typ::GenericParameter;
use ecow::EcoString;
use indexmap::IndexMap;

/// Represents a stack-based context for managing generic type parameters
/// during type inference and hydration.
///
/// The `Generics` structure maintains a stack of generic scopes.
/// Each scope (a `HashMap<usize>`) contains the names of generic type
/// parameters that are currently in scope and their ID's.
///
/// This allows the type checker and hydrator to correctly resolve
/// generic type names to their bound variables in nested or shadowed contexts.
///
/// # Fields
///
/// - `stack: Vec<HashMap<EcoString, usize>>` — A stack of scopes,
///   where each scope holds the names of generics currently active within that scope.
///
/// - `last_generic_id: usize` — Last generic id, used to
///   generate fresh UID's for generics.
///
/// # Example
///
/// ```rust
/// let mut generics = Generics::new();
/// generics.push_scope(vec!["T".into(), "U".into()]);
/// generics.push_scope(vec!["V".into()]);
///
/// // Top of stack: ["V"]
/// // Second scope: ["T", "U"]
///
/// generics.pop_scope(); // exits scope with ["V"]
/// ```
///
/// # Notes
/// - contains method checks generic name
///   only in the last scope.
///
#[derive(Default)]
pub struct Generics {
    stack: Vec<IndexMap<EcoString, usize>>,
    last_generic_id: usize,
}

/// Implementation
impl Generics {
    /// Pushes the scope onto the stack
    /// and inserts given generic arguments
    /// in it.
    ///
    /// # Parameters
    /// - `generics: Vec<EcoString>`
    ///   Generic parameter names.
    ///
    /// # Returns
    /// - `Vec<GenericParameter>`
    ///   Created generic parameters info
    ///
    pub fn push_scope(&mut self, generics: Vec<EcoString>) -> Vec<GenericParameter> {
        let generics: IndexMap<EcoString, usize> =
            generics.into_iter().map(|g| (g, self.fresh())).collect();
        self.stack.push(generics.clone());
        generics
            .into_iter()
            .map(|g| GenericParameter { name: g.0, id: g.1 })
            .collect()
    }

    /// Pushes a new scope consisting of **already constructed**
    /// generic parameters (usually reconstructed from a type).
    ///
    /// # Parameters
    /// - `generics: Vec<GenericParameter>`
    ///   Generic parameters.
    ///
    pub fn re_push_scope(&mut self, generics: Vec<GenericParameter>) {
        self.stack
            .push(generics.into_iter().map(|g| (g.name, g.id)).collect());
    }

    /// Pops scope from the stack
    pub fn pop_scope(&mut self) {
        self.stack.pop();
    }

    /// Returns generic ID by the name
    /// from the last scope, if generic exists.
    ///
    /// # Parameters
    /// - `name: &str`
    ///   Name of the generic
    ///
    pub fn get(&self, name: &str) -> Option<usize> {
        self.stack
            .last().and_then(|s| s.get(name).copied())
    }

    /// Generates fresh unique id
    /// for the generic type variable.
    ///
    #[inline]
    pub fn fresh(&mut self) -> usize {
        self.last_generic_id += 1;
        self.last_generic_id
    }
}
