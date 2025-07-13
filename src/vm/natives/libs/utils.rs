// импорты
use crate::lexer::address::Address;
use crate::vm::values::{Function, Instance, Native, Trait, Type, Unit, Value};
use crate::error;
use crate::errors::errors::Error;
use std::any::Any;

// ожидание инт
#[allow(unused)]
pub fn expect_int(addr: Address, value: Value, error: Option<Error>) ->i64 {
    if let Value::Int(i) = value {
        i
    } else {
        error!(error.unwrap_or(Error::own_text(
            addr.clone(),
            format!("expected i64, got {:?}", value),
            "check for types"
        )));
    }
}

// ожидание флоат
#[allow(unused)]
pub fn expect_float(addr: Address, value: Value, error: Option<Error>) ->f64 {
    if let Value::Float(f) = value {
        f
    } else {
        error!(error.unwrap_or(Error::own_text(
            addr.clone(),
            format!("expected f64, got {:?}", value),
            "check for types"
        )));
    }
}

// ожидание бул
#[allow(unused)]
pub fn expect_bool(addr: Address, value: Value, error: Option<Error>) ->bool {
    if let Value::Bool(b) = value {
        b
    } else {
        error!(error.unwrap_or(Error::own_text(
            addr.clone(),
            format!("expected bool, got {:?}", value),
            "check for types"
        )));
    }
}

// ожидание инстанса
#[allow(unused)]
pub fn expect_instance(addr: Address, value: Value, error: Option<Error>) ->*mut Instance {
    if let Value::Instance(i) = value {
        i
    } else {
        error!(error.unwrap_or(Error::own_text(
            addr.clone(),
            format!("expected instance, got {:?}", value),
            "check for types"
        )));
    }
}

// ожидание юнита
#[allow(unused)]
pub fn expect_unit(addr: Address, value: Value, error: Option<Error>) ->*mut Unit {
    if let Value::Unit(u) = value {
        u
    } else {
        error!(error.unwrap_or(Error::own_text(
            addr.clone(),
            format!("expected unit, got {:?}", value),
            "check for types"
        )));
    }
}

// ожидание трейта
#[allow(unused)]
pub fn expect_trait(addr: Address, value: Value, error: Option<Error>) ->*mut Trait {
    if let Value::Trait(t) = value {
        t
    } else {
        error!(error.unwrap_or(Error::own_text(
            addr.clone(),
            format!("expected trait, got {:?}", value),
            "check for types"
        )));
    }
}

// ожидание типа
#[allow(unused)]
pub fn expect_type(addr: Address, value: Value, error: Option<Error>) ->*mut Type {
    if let Value::Type(t) = value {
        t
    } else {
        error!(error.unwrap_or(Error::own_text(
            addr.clone(),
            format!("expected type, got {:?}", value),
            "check for types"
        )));
    }
}

// ожидание функции
#[allow(unused)]
pub fn expect_fn(addr: Address, value: Value, error: Option<Error>) ->*mut Function {
    if let Value::Fn(f) = value {
        f
    } else {
        error!(error.unwrap_or(Error::own_text(
            addr.clone(),
            format!("expected fn, got {:?}", value),
            "check for types"
        )));
    }
}

// ожидание нативной функции
#[allow(unused)]
pub fn expect_native(addr: Address, value: Value, error: Option<Error>) ->*mut Native {
    if let Value::Native(n) = value {
        n
    } else {
        error!(error.unwrap_or(Error::own_text(
            addr.clone(),
            format!("expected native, got {:?}", value),
            "check for types"
        )));
    }
}

// ожидание any
#[allow(unused)]
pub fn expect_any(addr: Address, value: Value, error: Option<Error>) ->*mut dyn Any {
    if let Value::Any(a) = value {
        a
    } else {
        error!(error.unwrap_or(Error::own_text(
            addr.clone(),
            format!("expected any, got {:?}", value),
            "check for types"
        )));
    }
}

// ожидание строки
#[allow(unused)]
pub fn expect_string(addr: Address, value: Value, error: Option<Error>) -> *const String {
    if let Value::String(s) = value {
        s
    } else {
        error!(error.unwrap_or(Error::own_text(
            addr.clone(),
            format!("expected string, got {:?}", value),
            "check for types"
        )));
    }
}

// ожидание списка
#[allow(unused)]

pub fn expect_list(addr: Address, value: Value, error: Option<Error>) ->*mut Vec<Value> {
    if let Value::List(l) = value {
        l
    } else {
        error!(error.unwrap_or(Error::own_text(
            addr.clone(),
            format!("expected list, got {:?}", value),
            "check for types"
        )));
    }
}