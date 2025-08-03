// imports
use crate::bytecode::Chunk;
use crate::flow::ControlFlow;
use crate::memory::memory;
use crate::table::Table;
use crate::vm::{VM, try_free_table};
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use watt_common::address::Address;

/// Module structure
///
/// `static`, `singleton` and `global`
/// instance-like object
///
#[derive(Clone, Debug)]
#[allow(unused)]
pub struct Module {
    pub table: *mut Table,
}
/// Unit implementation
impl Module {
    pub fn new(table: *mut Table) -> Module {
        Module { table }
    }
}
/// Unit drop implementation
impl Drop for Module {
    fn drop(&mut self) {
        unsafe {
            try_free_table(self.table);
        }
    }
}

/// Type structure
///
/// Type is a `instruction` to build
/// an instance
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Type {
    pub name: String,
    pub constructor: Vec<String>,
    pub body: *const Chunk,
    pub impls: Vec<String>,
    pub defined_in: *mut Table,
}
/// Type implementation
impl Type {
    /// New type
    pub fn new(
        name: String,
        constructor: Vec<String>,
        body: *const Chunk,
        impls: Vec<String>,
        defined_in: *mut Table,
    ) -> Type {
        Type {
            name,
            constructor,
            body,
            impls,
            defined_in,
        }
    }
}
/// Type drop implementation
impl Drop for Type {
    fn drop(&mut self) {
        memory::free_const_value(self.body);
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
    /// type, instance related to
    pub t: *mut Type,
    /// instance fields table
    pub fields: *mut Table,
}
/// Instance implementation
impl Instance {
    pub fn new(t: *mut Type, fields: *mut Table) -> Instance {
        Instance { t, fields }
    }
}
/// Instance drop implementation
impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            try_free_table(self.fields);
        }
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
    pub name: String,
    pub fields: *mut Table,
    pub defined_in: *mut Table,
}
/// Unit implementation
impl Unit {
    pub fn new(name: String, fields: *mut Table, defined_in: *mut Table) -> Unit {
        Unit {
            name,
            fields,
            defined_in,
        }
    }
}
/// Unit drop implementation
impl Drop for Unit {
    fn drop(&mut self) {
        unsafe {
            try_free_table(self.fields);
        }
    }
}

/// Default trait fn realisation
#[derive(Clone, Debug)]
pub struct DefaultTraitFn {
    pub params: Vec<String>,
    pub chunk: Chunk,
}
/// Default trait fn implementation
impl DefaultTraitFn {
    /// New default trait fn
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
    pub name: String,
    pub params_amount: usize,
    pub default: Option<DefaultTraitFn>,
}
/// Trait function implementation
impl TraitFn {
    /// New trait fn
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
    pub name: String,
    pub functions: Vec<TraitFn>,
}
/// Trait implementation
impl Trait {
    pub fn new(name: String, functions: Vec<TraitFn>) -> Trait {
        Trait { name, functions }
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
    Unit(*mut Unit),
    Instance(*mut Instance),
    Module(*mut Module),
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
    pub name: String,
    pub body: *const Chunk,
    pub params: Vec<String>,
    pub owner: Option<FnOwner>,
    pub closure: *mut Table,
}
/// Function implementation
impl Function {
    /// New function
    pub fn new(name: String, body: *const Chunk, params: Vec<String>) -> Function {
        Function {
            name,
            body,
            params,
            owner: None,
            closure: std::ptr::null_mut(),
        }
    }
}
/// Function drop implementation
impl Drop for Function {
    fn drop(&mut self) {
        unsafe {
            if !self.closure.is_null() {
                (*self.closure).captures -= 1;
                try_free_table(self.closure);
            }
        }
        memory::free_const_value(self.body);
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
    pub name: String,
    pub params_amount: usize,
    pub function: fn(&mut VM, Address, bool, *mut Table) -> Result<(), ControlFlow>,
    pub defined_in: *mut Table,
}
/// Native implementation
impl Native {
    /// New native
    pub fn new(
        name: String,
        params_amount: usize,
        function: fn(&mut VM, Address, bool, *mut Table) -> Result<(), ControlFlow>,
        defined_in: *mut Table,
    ) -> Native {
        Native {
            name,
            params_amount,
            function,
            defined_in,
        }
    }
}

/// Value
#[derive(Clone, Copy)]
pub enum Value {
    Float(f64),
    Int(i64),
    String(*const String),
    Bool(bool),
    Type(*mut Type),
    Fn(*mut Function),
    Native(*mut Native),
    Instance(*mut Instance),
    Unit(*mut Unit),
    Trait(*mut Trait),
    List(*mut Vec<Value>),
    Any(*mut dyn std::any::Any),
    Module(*mut Module),
    Null,
}
/// Debug implementation for value
impl Debug for Value {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe {
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
                    write!(fmt, "Fn{:?}", (**f).name)
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
}
/// Display implementation for value
impl Display for Value {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Instance(i) => unsafe {
                write!(fmt, "Instance{:?} of {:?}", *i, (*(**i).t).name)
            },
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
        match (*self, *other) {
            (Value::Instance(a), Value::Instance(b)) => unsafe { a == b },
            (Value::Fn(a), Value::Fn(b)) => unsafe { a == b },
            (Value::Native(a), Value::Native(b)) => unsafe { a == b },
            (Value::Bool(a), Value::Bool(b)) => unsafe { a == b },
            (Value::Type(a), Value::Type(b)) => unsafe { a == b },
            (Value::String(a), Value::String(b)) => unsafe { a == b },
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Unit(a), Value::Unit(b)) => unsafe { a == b },
            (Value::Trait(a), Value::Trait(b)) => unsafe { a == b },
            (Value::List(a), Value::List(b)) => unsafe { a == b },
            (Value::Any(a), Value::Any(b)) => unsafe { std::ptr::addr_eq(a, b) },
            _ => false,
        }
    }
}
/// Eq implementation for value
impl Eq for Value {}
/// Hash implementation for value
impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match *self {
            Value::Instance(a) => {
                (a as usize).hash(state);
            }
            Value::Fn(a) => {
                (a as usize).hash(state);
            }
            Value::Native(a) => {
                (a as usize).hash(state);
            }
            Value::Bool(a) => {
                a.hash(state);
            }
            Value::Type(a) => {
                (a as usize).hash(state);
            }
            Value::String(a) => {
                (a as usize).hash(state);
            }
            Value::Int(a) => {
                a.hash(state);
            }
            Value::Float(a) => {
                a.to_bits().hash(state);
            }
            Value::Unit(a) => {
                (a as usize).hash(state);
            }
            Value::Trait(a) => {
                (a as usize).hash(state);
            }
            Value::List(a) => {
                (a as usize).hash(state);
            }
            Value::Any(a) => {
                let any_ptr = a as *const () as usize;
                any_ptr.hash(state);
            }
            Value::Module(m) => (m as usize).hash(state),
            Value::Null => {
                0.hash(state);
            }
        }
    }
}
