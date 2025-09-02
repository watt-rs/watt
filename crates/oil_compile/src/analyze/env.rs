/// Imports
use crate::analyze::{
    errors::AnalyzeError,
    rc_ptr::RcPtr,
    typ::{Typ, Type},
};
use ecow::EcoString;
use miette::NamedSource;
use oil_common::{address::Address, bail};
use std::{cell::RefCell, collections::HashMap, sync::Arc};

/// Environment type
#[derive(PartialEq)]
pub enum EnvironmentType {
    Function(Typ),
    Loop,
    Conditional,
    ConstructorParams,
    Fields,
    Type(RcPtr<RefCell<Type>>),
    Module,
}

/// Environment
type Environment = (EnvironmentType, HashMap<EcoString, Typ>);

/// Environments stack
pub struct EnvironmentsStack {
    stack: Vec<Environment>,
}

/// Environments stack implementation
impl EnvironmentsStack {
    /// Creates new stack
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    /// Pushes environment
    pub fn push(&mut self, environment_type: EnvironmentType) {
        self.stack.push((environment_type, HashMap::new()))
    }

    /// Pops environment
    pub fn pop(&mut self) -> Option<Environment> {
        self.stack.pop()
    }

    /// Defines variable
    pub fn define(
        &mut self,
        named_source: &NamedSource<Arc<String>>,
        address: &Address,
        name: &EcoString,
        variable: Typ,
    ) {
        match self.stack.last_mut() {
            Some(env) => {
                if !env.1.contains_key(name) {
                    env.1.insert(name.clone(), variable);
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
        named_source: &NamedSource<Arc<String>>,
        address: &Address,
        name: &EcoString,
    ) -> Typ {
        for env in self.stack.iter().rev() {
            if env.1.contains_key(name) {
                return env.1.get(name).unwrap().clone();
            }
        }
        bail!(AnalyzeError::CouldNotResolve {
            src: named_source.clone(),
            span: address.span.clone().into(),
            name: name.clone()
        })
    }

    /// Checks variable existence
    pub fn exists(&self, name: &EcoString) -> bool {
        for env in self.stack.iter().rev() {
            if env.1.contains_key(name) {
                return true;
            }
        }
        false
    }

    /// Checks env with provided env type exists in hierarchy
    pub fn contains_env(&self, t: EnvironmentType) -> bool {
        for env in self.stack.iter().rev() {
            if env.0 == t {
                return true;
            }
        }
        false
    }

    /// Checks function exists in hierarchy
    pub fn contains_function(&self) -> Option<&Typ> {
        for env in self.stack.iter().rev() {
            match &env.0 {
                EnvironmentType::Function(typ) => return Some(typ),
                _ => continue,
            }
        }
        None
    }

    /// Checks type exists in hierarchy
    pub fn contains_type(&self) -> Option<&RcPtr<RefCell<Type>>> {
        for env in self.stack.iter().rev() {
            match &env.0 {
                EnvironmentType::Type(typ) => return Some(typ),
                _ => continue,
            }
        }
        None
    }
}
