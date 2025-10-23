/// Imports
use crate::{
    errors::TypeckError,
    typ::{Typ, Type},
};
use ecow::EcoString;
use miette::NamedSource;
use std::{cell::RefCell, collections::HashMap, sync::Arc};
use watt_common::{address::Address, bail, rc_ptr::RcPtr};

/// Rib kind
#[derive(PartialEq)]
pub enum RibKind {
    Function,
    Loop,
    Conditional,
    ConstructorParams,
    Fields,
    Pattern,
    Type(RcPtr<RefCell<Type>>),
}

/// Rib
pub type Rib = (RibKind, HashMap<EcoString, Typ>);

/// Ribs stack
pub struct RibsStack {
    stack: Vec<Rib>,
}

/// Ribs stack implementation
impl RibsStack {
    /// Creates new stack
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    /// Pushes environment
    pub fn push(&mut self, environment_type: RibKind) {
        self.stack.push((environment_type, HashMap::new()))
    }

    /// Pops environment
    pub fn pop(&mut self) -> Option<Rib> {
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
                    bail!(TypeckError::VariableIsAlreadyDefined {
                        src: named_source.clone(),
                        span: address.span.clone().into()
                    })
                }
            }
            None => todo!(),
        }
    }

    /// Defines variable.
    /// If definition exists, checks types equality.
    pub fn redefine(
        &mut self,
        named_source: &NamedSource<Arc<String>>,
        address: &Address,
        name: &EcoString,
        variable: Typ,
    ) {
        match self.stack.last_mut() {
            Some(env) => match env.1.get(name) {
                Some(def) => {
                    if def != &variable {
                        bail!(TypeckError::TypesMissmatch {
                            src: named_source.clone(),
                            span: address.span.clone().into(),
                            expected: def.clone(),
                            got: variable
                        })
                    }
                }
                None => {
                    env.1.insert(name.clone(), variable);
                }
            },
            None => todo!(),
        }
    }

    /// Lookups variable
    pub fn lookup(&self, name: &EcoString) -> Option<Typ> {
        for env in self.stack.iter().rev() {
            if env.1.contains_key(name) {
                return Some(env.1.get(name).unwrap().clone());
            }
        }
        None
    }

    /// Checks rib with provided env type exists in hierarchy
    pub fn contains_rib(&self, t: RibKind) -> bool {
        for env in self.stack.iter().rev() {
            if env.0 == t {
                return true;
            }
        }
        false
    }

    /// Checks type exists in hierarchy
    pub fn contains_type(&self) -> Option<&RcPtr<RefCell<Type>>> {
        for env in self.stack.iter().rev() {
            match &env.0 {
                RibKind::Type(typ) => return Some(typ),
                _ => continue,
            }
        }
        None
    }
}
