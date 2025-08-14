/// Imports
use ecow::EcoString;
use miette::NamedSource;
use oil_common::{address::Address, bail};
use std::collections::HashMap;
use crate::analyze::{analyze::Typ, errors::AnalyzeError};

/// Variables environment
#[derive(Debug, Clone)]
pub struct Environment {
    variables: HashMap<EcoString, Typ>,
}
/// Variables environment implementation
impl Environment {
    /// Creates new environment
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    /// Defines variable
    pub fn define(
        &mut self,
        named_source: &NamedSource<String>,
        address: &Address,
        name: EcoString,
        typ: Typ,
    ) {
        if !self.variables.contains_key(&name) {
            self.variables.insert(name, typ);
        } else {
            bail!(AnalyzeError::VariableIsAlreadyDefined {
                src: named_source.clone(),
                span: address.span.clone().into()
            })
        }
    }

    /// Stores variable
    pub fn store(
        &mut self,
        named_source: &NamedSource<String>,
        address: &Address,
        name: EcoString,
        typ: Typ,
    ) {
        if self.variables.contains_key(&name) {
            self.variables.insert(name, typ);
        } else {
            bail!(AnalyzeError::VariableIsNotDefined {
                src: named_source.clone(),
                span: address.span.clone().into()
            })
        }
    }

    /// Lookups variable
    pub fn lookup(
        &mut self,
        named_source: &NamedSource<String>,
        address: &Address,
        name: EcoString,
    ) -> Typ {
        if self.variables.contains_key(&name) {
            return self.variables.get(&name).unwrap().clone();
        } else {
            bail!(AnalyzeError::VariableIsNotDefined {
                src: named_source.clone(),
                span: address.span.clone().into()
            })
        }
    }

    /// Checks variable existence
    pub fn exists(&mut self, name: EcoString) -> bool {
        self.variables.contains_key(&name)
    }
}
