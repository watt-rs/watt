use oil_common::{address::Address, error, errors::Error};

//// Imports.
use crate::{
    call_stack::environment::{Environment, EnvironmentStack},
    mark,
    memory::{
        gc::Gc,
        trace::{Trace, Tracer},
    },
    values::Value,
};

/// The call frame.
#[derive(Debug)]
pub struct CallFrame {
    /// Environment stack.
    pub closure_environment: Option<Gc<Environment>>,
    pub environments: Gc<EnvironmentStack>,
}

/// Call frame implementation.
impl CallFrame {
    /// Creates new `CallFrame` with one environment.
    pub fn new() -> Self {
        Self {
            closure_environment: None,
            environments: Gc::new(EnvironmentStack::new()),
        }
    }

    /// Creates new `CallFrame` with one environment and closure environment set.
    pub fn with_closure(closure_environment: Gc<Environment>) -> Self {
        Self {
            closure_environment: Some(closure_environment),
            environments: Gc::new(EnvironmentStack::new()),
        }
    }

    /// Pushes environment to environments stack
    pub fn push(&mut self, environment: Gc<Environment>) {
        self.environments.push(environment);
    }

    /// Pops environment from environments stack
    pub fn pop(&mut self) -> Gc<Environment> {
        self.environments.pop()
    }

    /// Peeks environment from environments stack by ref
    pub fn peek(&self) -> Gc<Environment> {
        match self.environments.stack.front() {
            Some(env) => env.clone(),
            None => panic!("environments stack is empty. report this error to the developer."),
        }
    }

    /// Defines variable in last environment
    pub fn define(&mut self, address: &Address, name: &str, value: Value) {
        self.environments.define(address, name, value);
    }

    /// Stores variable in last environment or module
    pub fn store(&mut self, address: &Address, name: &str, value: Value) {
        if self.environments.is_exists(name) {
            self.environments.store(address, name, value);
        } else {
            match self.closure_environment.clone() {
                Some(mut environment) => environment.store(address, name, value),
                None => error!(Error::own_text(
                    address.clone(),
                    format!("variable {name} is not defined."),
                    "you should use `let` to declare variable."
                )),
            }
        }
    }

    /// Loads variable from last environment or module
    pub fn load(&mut self, address: &Address, name: &str) -> Value {
        if self.environments.is_exists(name) {
            self.environments.load(address, name)
        } else {
            match self.closure_environment.clone() {
                Some(environment) => environment.load(address, name),
                None => error!(Error::own_text(
                    address.clone(),
                    format!("variable {name} is not defined."),
                    "you should use `let` to declare variable."
                )),
            }
        }
    }

    /// Deletes variable from last environment or module
    pub fn delete(&mut self, address: &Address, name: &str) {
        if self.environments.is_exists(name) {
            self.environments.delete(address, name)
        } else {
            match self.closure_environment.clone() {
                Some(mut environment) => environment.delete(address, name),
                None => error!(Error::own_text(
                    address.clone(),
                    format!("variable {name} is not defined."),
                    "you should use `let` to declare variable."
                )),
            }
        }
    }

    /// Checks variable existence
    pub fn is_exists(&self, name: &str) -> bool {
        if self.environments.is_exists(name) {
            true
        } else {
            if let Some(environment) = &self.closure_environment {
                environment.is_exists(name)
            } else {
                false
            }
        }
    }
}

/// Trace implementation
impl Trace for CallFrame {
    unsafe fn trace(&self, tracer: &mut Tracer) {
        if let Some(module_environment) = &self.closure_environment {
            mark!(tracer, module_environment);
        }
        mark!(tracer, self.environments)
    }
}
