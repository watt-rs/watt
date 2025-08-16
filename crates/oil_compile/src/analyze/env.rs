/// Imports
use crate::analyze::{analyze::Typ, errors::AnalyzeError};
use ecow::EcoString;
use miette::NamedSource;
use oil_common::{address::Address, bail};
use std::collections::HashMap;

/// Environments stack
pub struct EnvironmentsStack {
    stack: Vec<HashMap<EcoString, Typ>>,
}

/// Environments stack implementation
impl EnvironmentsStack {
    /// Creates new stack
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    /// Pushes environment
    pub fn push(&mut self) {
        self.stack.push(HashMap::new())
    }

    /// Pops environment
    pub fn pop(&mut self) -> Option<HashMap<EcoString, Typ>> {
        self.stack.pop()
    }

    /// Defines variable
    pub fn define(
        &mut self,
        named_source: &NamedSource<String>,
        address: &Address,
        name: &EcoString,
        variable: Typ,
    ) {
        match self.stack.last_mut() {
            Some(env) => {
                if !env.contains_key(name) {
                    env.insert(name.clone(), variable);
                } else {
                    bail!(AnalyzeError::VariableIsAlreadyDefined {
                        src: named_source.clone(),
                        span: address.span.clone().into()
                    })
                }
            }
            None => todo!(),
        }
    }

    /// Lookups variable
    pub fn lookup(
        &self,
        named_source: &NamedSource<String>,
        address: &Address,
        name: &EcoString,
    ) -> Typ {
        for env in self.stack.iter().rev() {
            if env.contains_key(name) {
                return env.get(name).unwrap().clone();
            }
        }
        bail!(AnalyzeError::VariableIsNotDefined {
            src: named_source.clone(),
            span: address.span.clone().into()
        })
    }

    /// Tries to lookup variable
    pub fn try_lookup(&self, name: &EcoString) -> Option<Typ> {
        for env in self.stack.iter().rev() {
            if env.contains_key(name) {
                return env.get(name).map(|t| t.clone());
            }
        }
        None
    }

    /// Checks variable existence
    pub fn exists(&self, name: &EcoString) -> bool {
        for env in self.stack.iter().rev() {
            if env.contains_key(name) {
                return true;
            }
        }
        false
    }
}
