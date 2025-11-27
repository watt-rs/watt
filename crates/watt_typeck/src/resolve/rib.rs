/// Imports
use crate::{errors::TypeckError, typ::typ::Typ};
use ecow::EcoString;
use std::collections::HashMap;
use watt_common::{address::Address, bail};

/// A single lexical scope that mappings
/// variable names to their types.
///
/// `Rib` represents a single environment scope, storing variables
/// declared within that scope. Each `Rib` is typically pushed onto the
/// `RibsStack` when entering a new block, function.
///
/// # Important
/// - New rib isn't created during `Enum` or `Struct` analysys.
///
pub type Rib = HashMap<EcoString, Typ>;

/// Stack of lexical scopes (ribs).
///
/// `RibsStack` manages nested lexical scopes in a module.
/// Each function, new block, etc. scope is represented by a `Rib`.
/// The stack structure ensures that variable shadowing, scope exit,
/// and lookups are handled correctly.
///
/// # Important
/// - New rib isn't created during `Enum` or `Struct` analysys.
///
#[derive(Default)]
pub struct RibsStack {
    stack: Vec<Rib>,
}

/// Implementation
impl RibsStack {
    /// Pushes a new, empty rib onto the stack.
    ///
    /// Use this when entering a new lexical scope, such
    /// as a function body, new block, etc.
    ///
    /// # Important
    /// - New rib isn't created during `Enum` or `Struct` analysys.
    ///
    pub fn push(&mut self) {
        self.stack.push(HashMap::new())
    }

    /// Pops the top rib out the stack.
    ///
    /// Returns the popped `Rib` if the stack was not empty.
    /// Popping a rib removes this rib from the stack.
    ///
    /// Used to exit the scope.
    ///
    pub fn pop(&mut self) -> Option<Rib> {
        self.stack.pop()
    }

    /// Defines a variable in the current scope.
    ///
    /// # Parameters
    /// - `address`: The source location of the variable, used for error reporting.
    /// - `name`: The variable name.
    /// - `typ`: The type of the variable.
    /// - `redefine`: If true, overwrite any existing variable in the current scope.
    ///
    /// # Behavior
    /// - If `redefine` is false and the variable already exists in the current scope,
    ///   raises `TypeckError::VariableIsAlreadyDefined`.
    /// - Otherwise, inserts or overwrites the variable in the current scope.
    ///
    pub fn define(&mut self, address: &Address, name: &EcoString, typ: Typ, redefine: bool) {
        match self.stack.last_mut() {
            Some(env) => {
                if redefine || !env.contains_key(name) {
                    env.insert(name.clone(), typ);
                } else {
                    bail!(TypeckError::VariableIsAlreadyDefined {
                        src: address.source.clone(),
                        span: address.span.clone().into()
                    })
                }
            }
            None => todo!("No active scope to define variable"),
        }
    }

    /// Redefines a variable in the current scope with type checking.
    ///
    /// # Parameters
    /// - `address`: Source location of the variable.
    /// - `name`: The variable name.
    /// - `variable`: The new type of the variable.
    ///
    /// # Behavior
    /// - If the variable exists, checks that the new type equals the existing type.
    ///   If not, raises `TypeckError::TypesMissmatch`.
    /// - If the variable does not exist, inserts it into the current scope.
    ///
    pub fn redefine(&mut self, address: &Address, name: &EcoString, variable: Typ) {
        match self.stack.last_mut() {
            Some(env) => match env.get(name) {
                Some(def) => {
                    if def != &variable {
                        bail!(TypeckError::TypesMissmatch {
                            src: address.source.clone(),
                            span: address.span.clone().into(),
                            expected: def.clone(),
                            got: variable
                        })
                    }
                }
                None => {
                    env.insert(name.clone(), variable);
                }
            },
            None => todo!("No active scope to redefine variable"),
        }
    }

    /// Looks up a variable by name, searching from innermost to outermost scope.
    ///
    /// # Parameters
    /// - `name`: The variable name to lookup.
    ///
    /// # Returns
    /// - `Some(Typ)` if the variable is found in any scope.
    /// - `None` if the variable does not exist in any active scope.
    ///
    /// # Behavior
    /// - Iterates the `stack` in reverse order to respect lexical scoping,
    ///   ensuring that inner scopes shadow outer ones.
    ///
    pub fn lookup(&self, name: &EcoString) -> Option<Typ> {
        for env in self.stack.iter().rev() {
            if env.contains_key(name) {
                return Some(env.get(name).unwrap().clone());
            }
        }
        None
    }
}
