//// Imports.
use crate::{
    mark,
    memory::{
        gc::Gc,
        trace::{Trace, Tracer, mark_fx_hashmap},
    },
    values::Value,
};
use oil_common::{address::Address, error, errors::Error};
use rustc_hash::FxHashMap;
use std::collections::VecDeque;

/// Environment.
///
/// The variables container,
/// used as lexical scope and
/// table of fields in instances.
///
#[derive(Debug)]
pub struct Environment {
    /// Variables map
    pub variables: FxHashMap<String, Value>,
}

/// Environment implementation.
impl Environment {
    /// Creates new `Environment`.
    pub fn new() -> Self {
        Self {
            variables: FxHashMap::default(),
        }
    }

    /// Defines variable, if not exists.
    ///
    /// raises error, if variable
    /// is already defined in this environment.
    ///
    pub fn define(&mut self, address: &Address, name: &str, value: Value) {
        // Checking variable is already exists.
        if self.variables.contains_key(name) {
            error!(Error::own_text(
                address.clone(),
                format!("variable {name} is already defined."),
                "you should use another name to declare variable."
            ))
        }
        // Else, declaring variable.
        else {
            self.variables.insert(name.to_string(), value);
        }
    }

    /// Stores variable value, if variable
    /// already exists.
    ///
    /// Raises error, if variable
    /// is not defined in this environment.
    ///
    pub fn store(&mut self, address: &Address, name: &str, value: Value) {
        // Checking variable is already exists.
        if !self.variables.contains_key(name) {
            error!(Error::own_text(
                address.clone(),
                format!("variable {name} is not defined."),
                "you should use `let` to declare variable."
            ))
        }
        // Else, declaring variable.
        else {
            self.variables.insert(name.to_string(), value);
        }
    }

    /// Loads variable value, if
    /// variable is exists.
    ///
    /// Raises error, if variable
    /// is not defined in this environment.
    ///
    pub fn load(&self, address: &Address, name: &str) -> Value {
        // Matching fetch result.
        match self.variables.get(name) {
            // If variable found, returning it's value.
            Some(value) => return value.clone(),
            // Else, raising an error.
            None => error!(Error::own_text(
                address.clone(),
                format!("variable {name} is not defined."),
                "you should use `let` to declare variable."
            )),
        }
    }

    /// Checks variable with `name` exists
    pub fn is_exists(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    /// Deletes variable from map,
    /// if variable exists.
    ///
    /// Raises error, if variable
    /// is not defined in this environment.
    pub fn delete(&mut self, address: &Address, name: &str) {
        // Checking variable is already exists.
        if !self.variables.contains_key(name) {
            error!(Error::own_text(
                address.clone(),
                format!("variable {name} is not defined."),
                "you should use `let` to declare variable."
            ));
        }
        // Else, deleting variable.
        else {
            self.variables.remove(name);
        }
    }
}

/// Trace implementation for environment.
impl Trace for Environment {
    unsafe fn trace(&self, tracer: &mut Tracer) {
        mark_fx_hashmap(tracer, &self.variables);
    }
}

/// Environment stack.
///
/// Represents the environment stack,
/// used for lexical environments.
///
#[derive(Debug)]
pub struct EnvironmentStack {
    /// Stack of environments.
    pub stack: VecDeque<Gc<Environment>>,
}

/// Environment stack implementation.
impl EnvironmentStack {
    /// Creates new environment stack.
    pub fn new() -> Self {
        return Self {
            stack: VecDeque::new(),
        };
    }

    /// Pushes environment to stack.
    pub fn push(&mut self, environment: Gc<Environment>) {
        // Pushing environment.
        self.stack.push_front(environment);
    }

    /// Pops environment from stack.
    pub fn pop(&mut self) -> Gc<Environment> {
        // Popping environment.
        match self.stack.pop_front() {
            Some(env) => env,
            None => {
                panic!("could not pop from environments stack. report this error to the developer.")
            }
        }
    }

    /// Defines variable is last environment. if variable
    /// isn't defined in the front environment.
    ///
    /// Raises error, if variable is already exists.
    /// Panics, if stack is empty.
    pub fn define(&mut self, address: &Address, name: &str, value: Value) {
        // Matching front.
        match self.stack.front_mut() {
            Some(front) => front.define(address, name, value),
            None => panic!(
                "could not define a variable in empty environment stack. report this error to the developer."
            ),
        }
    }

    /// Stores variable value, if variable exists
    /// in the `environemnts` in the `stack`.
    ///
    /// Raises error, if variable doesn't already exists.
    /// Panics, if stack is empty.
    pub fn store(&mut self, address: &Address, name: &str, value: Value) {
        // If stack isn't empty.
        if !self.stack.is_empty() {
            // Searching environment, where variable exists.
            for environment in &mut self.stack {
                // If variable exists in this variable, storing value.
                if environment.is_exists(name) {
                    environment.store(address, name, value);
                    return;
                }
            }
            // If not environment found with needed variable, raising error.
            error!(Error::own_text(
                address.clone(),
                format!("variable {name} is not defined."),
                "you should use `let` to declare variable."
            ));
        } else {
            panic!(
                "could not define a variable in empty environment stack. report this error to the developer.",
            );
        }
    }

    /// Loads variable value, if variable exists
    /// in the `front` environment.
    ///
    /// Raises error, if variable doesn't already exists.
    /// Panics, if stack is empty.
    pub fn load(&mut self, address: &Address, name: &str) -> Value {
        // If stack isn't empty.
        if !self.stack.is_empty() {
            // Searching environment, where variable exists.
            for environment in &mut self.stack {
                // If variable exists in this variable, loading value.
                if environment.is_exists(name) {
                    return environment.load(address, name);
                }
            }
            // If not environment found with needed variable, raising error.
            error!(Error::own_text(
                address.clone(),
                format!("variable {name} is not defined."),
                "you should use `let` to declare variable."
            ));
        } else {
            panic!(
                "could not define a variable in empty environment stack. report this error to the developer.",
            );
        }
    }

    /// Checks variable with `name` exists
    /// Panics, if stack is empty.
    pub fn is_exists(&self, name: &str) -> bool {
        // If stack isn't empty.
        if !self.stack.is_empty() {
            // Searching environment, where variable exists.
            for environment in &self.stack {
                // If variable exists in this variable, loading value.
                if environment.is_exists(name) {
                    return true;
                }
            }
            // If not environment found with needed variable.
            return false;
        } else {
            panic!(
                "could not define a variable in empty environment stack. report this error to the developer.",
            );
        }
    }

    /// Deletes variable from `front` environment.
    ///
    /// Raises error, if variable
    /// is not defined in environment.
    /// Panics, if stack is empty.
    pub fn delete(&mut self, address: &Address, name: &str) {
        // Matching front.
        match self.stack.front_mut() {
            Some(front) => front.delete(address, name),
            None => panic!(
                "could not define a variable in empty environment stack. report this error to the developer."
            ),
        }
    }
}

/// Trace implementation for environment stack.
impl Trace for EnvironmentStack {
    unsafe fn trace(&self, tracer: &mut Tracer) {
        for env in &self.stack {
            mark!(tracer, env);
        }
    }
}
