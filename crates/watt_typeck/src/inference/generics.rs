/// Imports
use ecow::EcoString;

/// Represents a stack-based context for managing generic type parameters
/// during type inference and hydration.
///
/// The `Generics` structure maintains a stack of generic scopes.
/// Each scope (a `Vec<EcoString>`) contains the names of generic type
/// parameters that are currently in scope.
///
/// This allows the type checker and hydrator to correctly resolve
/// generic type names to their bound variables in nested or shadowed contexts.
///
/// # Fields
///
/// - `stack: Vec<Vec<EcoString>>` — A stack of scopes, where each scope holds the names
///   of generics currently active within that scope.
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
    stack: Vec<Vec<EcoString>>,
}

/// Implementation
impl Generics {
    /// Pushes the scope onto the stack
    /// and inserts given generic arguments
    /// in it.
    pub fn push_scope(&mut self, generics: Vec<EcoString>) {
        self.stack.push(generics);
    }

    /// Pops scope from the stack
    pub fn pop_scope(&mut self) {
        self.stack.pop();
    }

    /// Checks last scope
    /// contains generic name
    pub fn contains(&self, name: &EcoString) -> bool {
        self.stack.last().map_or(false, |s| s.contains(name))
    }
}
