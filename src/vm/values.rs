use std::fmt::{Debug, Formatter};
use crate::lexer::address::Address;
use crate::vm::bytecode::Chunk;
use crate::vm::flow::ControlFlow;
use crate::vm::table::Table;
use crate::vm::vm::VM;

// символ
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub full_name: Option<String>,
}
impl Symbol {
    pub fn new(name: String, full_name: String) -> Symbol {
        Symbol {name, full_name: Option::Some(full_name)}
    }
    pub fn new_option(name: String, full_name: Option<String>) -> Symbol {
        Symbol {name, full_name }
    }
    pub fn by_name(name: String) -> Symbol {
        Symbol {name, full_name:Option::None}
    }
}

// тип
#[derive(Clone, Debug, Eq, PartialEq)] // todo check eq, partialeq
pub struct Type {
    pub name: Symbol,
    pub constructor: Vec<String>,
    pub body: *const Chunk
}
impl Type {
    pub fn new(name: Symbol, constructor: Vec<String>, body: *const Chunk) -> Type {
        Type {name, constructor, body}
    }
}

// экземпляр типа
#[derive(Clone, Debug)]
pub struct Instance {
    pub t: *mut Type,
    pub fields: *mut Table
}
impl Instance {
    pub fn new(t: *mut Type, fields: *mut Table) -> Instance {
        Instance {t, fields}
    }
}

// юнит
#[derive(Clone, Debug)]
pub struct Unit {
    pub name: Symbol,
    pub fields: *mut Table,
}
impl Unit {
    pub fn new(name: Symbol, fields: *mut Table) -> Unit {
        Unit {name, fields}
    }
}

// владелец функции
#[derive(Clone, Debug)]
pub enum FnOwner {
    Unit(*mut Unit),
    Instance(*mut Instance),
}

// функция
#[derive(Clone, Debug)]
pub struct Function {
    pub name: Symbol,
    pub body: *const Chunk,
    pub params: Vec<String>,
    pub owner: *mut FnOwner,
    pub closure: *mut Table
}
impl Function {
    pub fn new(name: Symbol, body: *const Chunk, params: Vec<String>) -> Function {
        Function {
            name,
            body,
            params,
            owner: std::ptr::null_mut(),
            closure: std::ptr::null_mut()
        }
    }
}

// нативная функция
#[derive(Clone, Debug)]
pub struct Native {
    pub name: Symbol,
    pub params_amount: usize,
    pub function: fn(&mut VM,Address,bool,*mut Table) -> Result<(), ControlFlow>
}
impl Native {
    pub fn new(name: Symbol, params_amount: usize, function: fn(&mut VM,Address,bool,*mut Table) -> Result<(), ControlFlow>) -> Native {
        Native {name, params_amount, function}
    }
}

// значение
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
    Null
}
impl Debug for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe {
            match self {
                Value::String(s) => {
                    write!(f, "{}", **s)
                },
                Value::Instance(i) => {
                    write!(f, "Instance{:?}", *i)
                },
                Value::Fn(fun) => {
                    write!(f, "Fn{:?}", *fun)
                },
                Value::Native(n) => {
                    write!(f, "Native{:?}", *n)
                },
                Value::Unit(n) => {
                    write!(f, "Unit{:?}", **n)
                },
                Value::Null => {
                    write!(f, "Null")
                },
                Value::Bool(b) => {
                    write!(f, "{}", *b)
                }
                Value::Type(t) => {
                    write!(f, "Type{:?}", **t)
                }
                Value::Int(i) => {
                    write!(f, "{}", *i)
                }
                Value::Float(fl) => {
                    write!(f, "{}", *fl)
                }
            }
        }
    }
}
impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        match (*self, *other) {
            (Value::Instance(a), Value::Instance(b)) => unsafe {
                a == b
            }
            (Value::Fn(a), Value::Fn(b)) => unsafe {
                a == b
            }
            (Value::Native(a), Value::Native(b)) => unsafe {
                a == b
            }
            (Value::Bool(a), Value::Bool(b)) => unsafe {
                a == b
            }
            (Value::Type(a), Value::Type(b)) => unsafe {
                a == b
            }
            (Value::String(a), Value::String(b)) => unsafe {
                **a == **b
            }
            (Value::Int(a), Value::Int(b)) => {
                a == b
            }
            (Value::Float(a), Value::Float(b)) => {
                a == b
            }
            (Value::Unit(a), Value::Unit(b)) => unsafe {
                a == b
            }
            _ => false
        }
    }
}