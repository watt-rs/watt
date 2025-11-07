/// Imports
use ecow::EcoString;

/// Generic rib
pub type GenericRib = Vec<EcoString>;

/// Generic scopes stack
#[derive(Default)]
pub struct Generics {
    /// Last generic id
    last_generic_id: usize,
    /// Generics stack
    generics: Vec<GenericRib>,
}

/// Implementation
impl Generics {
    /// Enters scope
    pub fn enter(&mut self, generics: Vec<EcoString>) {
        // Pushing scope
        self.generics.push(generics);
    }

    /// Exits scope
    pub fn exit(&mut self) {
        self.generics.pop();
    }

    /// Cheks a generic existence
    pub fn contains(&self, name: &EcoString) -> bool {
        for rib in self.generics.iter().rev() {
            if rib.contains(&name) {
                return true;
            }
        }
        false
    }

    /// Generates fresh generic id
    pub fn fresh(&mut self) -> usize {
        self.last_generic_id += 1;
        self.last_generic_id
    }
}
