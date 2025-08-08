// imports
use crate::bytecode::Chunk;
use crate::call_stack::environment::Environment;
use crate::flow::ControlFlow;
use crate::mark;
use crate::memory::gc::Gc;
use crate::memory::trace::{Trace, Tracer};
use crate::vm::VirtualMachine;
use oil_common::address::Address;
use std::any::Any;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

/// Module structure
///
/// `static`, `singleton` and `global`
/// instance-like object
///
#[derive(Clone, Debug)]
#[allow(unused)]
pub struct Module {
    /// Module environment
    pub environment: Gc<Environment>,
}
/// Module implementation
impl Module {
    /// Creates mew module
    pub fn new(environment: Gc<Environment>) -> Module {
        Module { environment }
    }
}
/// Trace implementation for module
impl Trace for Module {
    unsafe fn trace(&self, tracer: &mut Tracer) {
        mark!(tracer, &self.environment);
    }
}

/// Type structure
///
/// Type is a `instruction` to build
/// an instance
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Type {
    /// Type name
    pub name: String,
    /// Type constructor
    pub constructor: Vec<String>,
    /// Body chunk
    pub body: Gc<Chunk>,
    /// Traits
    pub impls: Vec<Gc<Trait>>,
    /// Module environment, where type is defined
    pub module: Gc<Module>,
}
/// Type implementation
impl Type {
    /// Creates new type
    pub fn new(
        name: String,
        constructor: Vec<String>,
        body: Gc<Chunk>,
        impls: Vec<Gc<Trait>>,
        module: Gc<Module>,
    ) -> Type {
        Type {
            name,
            constructor,
            body,
            impls,
            module,
        }
    }
}
/// Trace implementation for type
impl Trace for Type {
    unsafe fn trace(&self, tracer: &mut Tracer) {
        mark!(tracer, &self.body);
        mark!(tracer, &self.module);
    }
}

/// Instance structure
///
/// Object created from `Type`
/// instructions
///
#[derive(Clone, Debug)]
#[allow(unused)]
pub struct Instance {
    /// Type, instance related to
    pub oil_type: Gc<Type>,
    /// Instance fields
    pub environment: Gc<Environment>,
}
/// Instance implementation
impl Instance {
    /// Creates new instance
    pub fn new(oil_type: Gc<Type>, environment: Gc<Environment>) -> Instance {
        Instance {
            oil_type,
            environment,
        }
    }
}
/// Trace implementation for instance
impl Trace for Instance {
    unsafe fn trace(&self, tracer: &mut Tracer) {
        mark!(tracer, &self.oil_type);
        mark!(tracer, &self.environment);
    }
}

/// Unit structure
///
/// `static`, `singleton` and `global`
/// instance-like object
///
#[derive(Clone, Debug)]
#[allow(unused)]
pub struct Unit {
    /// Unit name
    pub name: String,
    /// Unit fields
    pub environment: Gc<Environment>,
    /// Module environment, where unit is defined
    pub module: Gc<Module>,
}
/// Unit implementation
impl Unit {
    /// Creates new unit
    pub fn new(name: String, environment: Gc<Environment>, module: Gc<Module>) -> Unit {
        Unit {
            name,
            environment,
            module,
        }
    }
}
/// Trace implementation for unit
impl Trace for Unit {
    unsafe fn trace(&self, tracer: &mut Tracer) {
        mark!(tracer, &self.environment);
        mark!(tracer, &self.module);
    }
}

/// Default trait fn realisation
#[derive(Clone, Debug)]
pub struct DefaultTraitFn {
    /// Parameters
    pub params: Vec<String>,
    /// Chunk
    pub chunk: Chunk,
}
/// Default trait fn implementation
impl DefaultTraitFn {
    /// Creates new default trait fn
    pub fn new(params: Vec<String>, chunk: Chunk) -> DefaultTraitFn {
        DefaultTraitFn { params, chunk }
    }
}

/// Trait function
///
/// contains `name`, `params_amount` and
/// `optional` *default implementation*
///
#[derive(Clone, Debug)]
pub struct TraitFn {
    /// Name of trait fn
    pub name: String,
    /// Parameters amount
    pub params_amount: usize,
    /// Default implementation
    pub default: Option<DefaultTraitFn>,
}
/// Trait function implementation
impl TraitFn {
    /// Creates new trait fn
    pub fn new(name: String, params_amount: usize, default: Option<DefaultTraitFn>) -> TraitFn {
        TraitFn {
            name,
            params_amount,
            default,
        }
    }
}

/// Trait
///
/// This is an abstraction that
/// contains functions that a type
/// must implement.
///
/// Functions may contain a default implementation.
/// In this case, types that implement a trait may
/// not implement the function, but may override it,
/// but only with its signature.
///
#[derive(Clone, Debug)]
#[allow(unused)]
pub struct Trait {
    /// Trait name
    pub name: String,
    /// Trait functions
    pub functions: Vec<TraitFn>,
    /// Module environment, where trait is defined
    pub module: Gc<Module>,
}
/// Trait implementation
impl Trait {
    /// Creates new trait
    pub fn new(name: String, functions: Vec<TraitFn>, module: Gc<Module>) -> Trait {
        Trait {
            name,
            functions,
            module,
        }
    }
}
/// Trace implementation for trace
impl Trace for Trait {
    unsafe fn trace(&self, tracer: &mut Tracer) {
        mark!(tracer, self.module)
    }
}
/// Trait drop implementation
impl Drop for Trait {
    fn drop(&mut self) {
        for function in self.functions.drain(..) {
            drop(function);
        }
    }
}

/// Fn owner
///
/// Owner of a function, be it
/// a unit or instance.
///
#[derive(Clone, Debug)]
pub enum FnOwner {
    Unit(Gc<Unit>),
    Instance(Gc<Instance>),
}
/// Trace implementation for fn owner
impl Trace for FnOwner {
    unsafe fn trace(&self, tracer: &mut Tracer) {
        match self {
            FnOwner::Unit(unit) => mark!(tracer, unit),
            FnOwner::Instance(instance) => mark!(tracer, instance),
        }
    }
}

/// Function
///
/// Just a function that have name,
/// params, body, `closure`, `owner`
/// (something, that owns function, be it unit or instance)
///
#[derive(Clone, Debug)]
#[allow(unused)]
pub struct Function {
    /// Function name
    pub name: String,
    /// Body chunk
    pub body: Gc<Chunk>,
    /// Parameters
    pub params: Vec<String>,
    /// Function owner `unit`/`instance`
    pub owner: Option<Gc<FnOwner>>,
    /// Function closure
    pub closure: Gc<Environment>,
    /// Module environment, where function is defined
    pub module: Gc<Module>,
}
/// Function implementation
impl Function {
    /// New function
    pub fn new(
        name: String,
        body: Gc<Chunk>,
        params: Vec<String>,
        module: Gc<Module>,
        closure: Gc<Environment>,
    ) -> Function {
        Function {
            name,
            body,
            params,
            owner: None,
            closure,
            module,
        }
    }
}
/// Trace implementation for function
impl Trace for Function {
    unsafe fn trace(&self, tracer: &mut Tracer) {
        mark!(tracer, &self.body);
        mark!(tracer, &self.module);
        mark!(tracer, &self.closure);
        if let Some(owner) = &self.owner {
            mark!(tracer, &owner);
        }
    }
}

/// Native function
///
/// Function, that wrote in rust, but can
/// be used in Watt, for example: io@println
///
#[derive(Clone, Debug)]
#[allow(unused)]
pub struct Native {
    /// Native name
    pub name: String,
    /// Parameters amount
    pub params_amount: usize,
    /// Function
    pub function: fn(&mut VirtualMachine, Address, bool) -> Result<(), ControlFlow>,
}
/// Native implementation
impl Native {
    /// New native
    pub fn new(
        name: String,
        params_amount: usize,
        function: fn(&mut VirtualMachine, Address, bool) -> Result<(), ControlFlow>,
    ) -> Native {
        Native {
            name,
            params_amount,
            function,
        }
    }
}
/// Trace implementation for native
impl Trace for Native {
    unsafe fn trace(&self, _: &mut Tracer) {}
}

/// Value
#[derive(Clone)]
pub enum Value {
    Float(f64),
    Int(i64),
    String(Gc<String>),
    Bool(bool),
    Type(Gc<Type>),
    Fn(Gc<Function>),
    Native(Gc<Native>),
    Instance(Gc<Instance>),
    Unit(Gc<Unit>),
    Trait(Gc<Trait>),
    List(Gc<Vec<Value>>),
    Any(Gc<*mut dyn Any>),
    Module(Gc<Module>),
    Null,
}
/// Debug implementation for value
impl Debug for Value {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(s) => {
                write!(fmt, "{}", **s)
            }
            Value::Instance(i) => {
                write!(fmt, "Instance{:?}", *i)
            }
            Value::Trait(t) => {
                write!(fmt, "Trait{:?}", *t)
            }
            Value::Fn(f) => {
                write!(fmt, "Fn{:?}", f.name)
            }
            Value::Native(n) => {
                write!(fmt, "Native{:?}", *n)
            }
            Value::Unit(n) => {
                write!(fmt, "Unit{:?}", *n)
            }
            Value::Null => {
                write!(fmt, "Null")
            }
            Value::Bool(b) => {
                write!(fmt, "{}", *b)
            }
            Value::Type(t) => {
                write!(fmt, "Type{:?}", *t)
            }
            Value::Int(i) => {
                write!(fmt, "{}", *i)
            }
            Value::Float(fl) => {
                write!(fmt, "{}", *fl)
            }
            Value::List(l) => {
                write!(fmt, "List{:?}", *l)
            }
            Value::Any(a) => {
                write!(fmt, "Any{:?}", *a)
            }
            Value::Module(m) => {
                write!(fmt, "Module{:?}", *m)
            }
        }
    }
}
/// Display implementation for value
impl Display for Value {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Instance(i) => {
                write!(fmt, "Instance{:?} of {:?}", *i, i.oil_type.name)
            }
            _ => write!(fmt, "{self:?}"),
        }
    }
}
/// PartialEq implementation for value
///
/// Value types
///  compared by value,
/// Reference types
///  (`instance`, `type`, `fn`, `native`, `list`
///   `native`, `trait`, `any`, `unit`)
///  compared by pointer address
///
#[allow(unused_unsafe)]
impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        match (self.clone(), other.clone()) {
            (Value::Instance(a), Value::Instance(b)) => a == b,
            (Value::Fn(a), Value::Fn(b)) => a == b,
            (Value::Native(a), Value::Native(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Type(a), Value::Type(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Unit(a), Value::Unit(b)) => a == b,
            (Value::Trait(a), Value::Trait(b)) => a == b,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Any(a), Value::Any(b)) => a == b,
            _ => false,
        }
    }
}
/// Eq implementation for value
impl Eq for Value {}
/// Hash implementation for value
impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self.clone() {
            Value::Instance(a) => {
                a.hash(state);
            }
            Value::Fn(a) => {
                a.hash(state);
            }
            Value::Native(a) => {
                a.hash(state);
            }
            Value::Bool(a) => {
                a.hash(state);
            }
            Value::Type(a) => {
                a.hash(state);
            }
            Value::String(a) => {
                a.hash(state);
            }
            Value::Int(a) => {
                a.hash(state);
            }
            Value::Float(a) => {
                a.to_bits().hash(state);
            }
            Value::Unit(a) => {
                a.hash(state);
            }
            Value::Trait(a) => {
                a.hash(state);
            }
            Value::List(a) => {
                a.hash(state);
            }
            Value::Any(a) => {
                a.hash(state);
            }
            Value::Module(a) => a.hash(state),
            Value::Null => {
                0.hash(state);
            }
        }
    }
}
