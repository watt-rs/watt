use crate::memory::gc::Gc;
// imports
use crate::values::{Function, Instance, Native, Trait, Type, Unit, Value};
use std::any::Any;
use watt_common::address::Address;
use watt_common::{error, errors::Error};

/// Expects value is int, otherwise raises error
#[allow(unused)]
pub fn expect_int(addr: &Address, value: Value) -> i64 {
    if let Value::Int(i) = value {
        i
    } else {
        error!(Error::own_text(
            addr.clone(),
            format!("expected i64, got {value:?}"),
            "check for types"
        ));
    }
}

/// Expects value is float, otherwise raises error
#[allow(unused)]
pub fn expect_float(addr: &Address, value: Value) -> f64 {
    if let Value::Float(f) = value {
        f
    } else {
        error!(Error::own_text(
            addr.clone(),
            format!("expected f64, got {value:?}"),
            "check for types"
        ));
    }
}

/// Expects value is bool, otherwise raises error
#[allow(unused)]
pub fn expect_bool(addr: &Address, value: Value) -> bool {
    if let Value::Bool(b) = value {
        b
    } else {
        error!(Error::own_text(
            addr.clone(),
            format!("expected bool, got {value:?}"),
            "check for types"
        ));
    }
}

/// Expects value is instance, otherwise raises error
#[allow(unused)]
pub fn expect_instance(addr: &Address, value: Value) -> Gc<Instance> {
    if let Value::Instance(i) = value {
        i
    } else {
        error!(Error::own_text(
            addr.clone(),
            format!("expected instance, got {value:?}"),
            "check for types"
        ));
    }
}

/// Expects value is unit, otherwise raises error
#[allow(unused)]
pub fn expect_unit(addr: &Address, value: Value) -> Gc<Unit> {
    if let Value::Unit(u) = value {
        u
    } else {
        error!(Error::own_text(
            addr.clone(),
            format!("expected unit, got {value:?}"),
            "check for types"
        ));
    }
}

/// Expects value is trait, otherwise raises error
#[allow(unused)]
pub fn expect_trait(addr: &Address, value: Value) -> Gc<Trait> {
    if let Value::Trait(t) = value {
        t
    } else {
        error!(Error::own_text(
            addr.clone(),
            format!("expected trait, got {value:?}"),
            "check for types"
        ));
    }
}

/// Expects value is type, otherwise raises error
#[allow(unused)]
pub fn expect_type(addr: &Address, value: Value) -> Gc<Type> {
    if let Value::Type(t) = value {
        t
    } else {
        error!(Error::own_text(
            addr.clone(),
            format!("expected type, got {value:?}"),
            "check for types"
        ));
    }
}

/// Expects value is fn, otherwise raises error
#[allow(unused)]
pub fn expect_fn(addr: &Address, value: Value) -> Gc<Function> {
    if let Value::Fn(f) = value {
        f
    } else {
        error!(Error::own_text(
            addr.clone(),
            format!("expected fn, got {value:?}"),
            "check for types"
        ));
    }
}

/// Expects value is native, otherwise raises error
#[allow(unused)]
pub fn expect_native(addr: &Address, value: Value) -> Gc<Native> {
    if let Value::Native(n) = value {
        n
    } else {
        error!(Error::own_text(
            addr.clone(),
            format!("expected native, got {value:?}"),
            "check for types"
        ));
    }
}

/// Expects value is any, otherwise raises error
#[allow(unused)]
pub fn expect_any(addr: &Address, value: Value, error: Option<Error>) -> *mut dyn Any {
    if let Value::Any(a) = value {
        *a
    } else {
        error!(error.unwrap_or(Error::own_text(
            addr.clone(),
            format!("expected any, got {value:?}"),
            "check for types"
        )));
    }
}

/// Expects value is string, otherwise raises error
#[allow(unused)]
pub fn expect_string(addr: &Address, value: Value) -> Gc<String> {
    if let Value::String(s) = value {
        s
    } else {
        error!(Error::own_text(
            addr.clone(),
            format!("expected string, got {value:?}"),
            "check for types"
        ));
    }
}

/// Expects value is string, if it's a string returns cloned, otherwise raises error
#[allow(unused)]
pub unsafe fn expect_cloned_string(addr: &Address, value: Value) -> String {
    if let Value::String(s) = value {
        (*s).clone()
    } else {
        error!(Error::own_text(
            addr.clone(),
            format!("expected string, got {value:?}"),
            "check for types"
        ));
    }
}

/// Expects value is list, otherwise raises error
#[allow(unused)]
pub fn expect_list(addr: &Address, value: Value) -> Gc<Vec<Value>> {
    if let Value::List(l) = value {
        l
    } else {
        error!(Error::own_text(
            addr.clone(),
            format!("expected list, got {value:?}"),
            "check for types"
        ));
    }
}

/// Expects value is list of strings, otherwise raises error
pub unsafe fn expect_string_list(addr: &Address, value: Value) -> Vec<String> {
    if let Value::List(l) = value {
        let mut strings = vec![];
        for value in (*l).clone() {
            match value {
                Value::String(string) => {
                    strings.push((*string).clone());
                }
                _ => {
                    error!(Error::own_text(
                        addr.clone(),
                        format!("expected strings list, got {value:?}"),
                        "check for types"
                    ));
                }
            }
        }
        strings
    } else {
        error!(Error::own_text(
            addr.clone(),
            format!("expected strings list, got {value:?}"),
            "check for types"
        ));
    }
}
