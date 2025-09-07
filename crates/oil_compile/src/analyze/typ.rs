/// Imports
use crate::analyze::{rc_ptr::RcPtr, resolve::ModDef};
use ecow::EcoString;
use miette::NamedSource;
use oil_ast::ast::Publicity;
use oil_common::address::Address;
use std::{cell::RefCell, collections::HashMap, fmt::Debug, sync::Arc};

/// Prelude type
#[derive(Debug, Clone, PartialEq)]
pub enum PreludeType {
    Int,
    Float,
    Bool,
    String,
}

/// Custom type
#[derive(Clone)]
pub struct Type {
    pub source: NamedSource<Arc<String>>,
    pub location: Address,
    pub name: EcoString,
    pub params: Vec<Typ>,
    pub env: HashMap<EcoString, WithPublicity<Typ>>,
}

/// Debug implementation
impl Debug for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Custom({})", self.name)
    }
}

/// Enum varient
#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub location: Address,
    pub name: EcoString,
    pub params: HashMap<EcoString, Typ>,
}

/// Custom enum
#[derive(Clone)]
#[allow(dead_code)]
pub struct Enum {
    pub source: NamedSource<Arc<String>>,
    pub location: Address,
    pub name: EcoString,
    pub variants: Vec<EnumVariant>,
}

/// Debug implementation
impl Debug for Enum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Enum({})", self.name)
    }
}

/// Function
#[derive(Clone)]
pub struct Function {
    pub source: NamedSource<Arc<String>>,
    pub location: Address,
    pub name: EcoString,
    pub params: Vec<Typ>,
    pub ret: Typ,
}

/// Debug implementation
impl Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Function({})", self.name)
    }
}

/// Custom type
#[derive(Clone, PartialEq, Debug)]
pub enum CustomType {
    Enum(RcPtr<Enum>),
    Type(RcPtr<RefCell<Type>>),
}

/// Module
#[derive(Clone)]
#[allow(dead_code)]
pub struct Module {
    pub source: NamedSource<Arc<String>>,
    pub name: EcoString,
    pub fields: HashMap<EcoString, ModDef>,
}

/// Debug implementation
impl Debug for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Prelude({})", self.name)
    }
}

/// Typ
#[derive(Clone)]
pub enum Typ {
    Prelude(PreludeType),
    Custom(RcPtr<RefCell<Type>>),
    Enum(RcPtr<Enum>),
    Function(RcPtr<Function>),
    Dyn,
    Void,
}

/// PartialEq implementation
impl PartialEq for Typ {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Typ::Prelude(a), Typ::Prelude(b)) => a == b,
            (Typ::Custom(a), Typ::Custom(b)) => a == b,
            (Typ::Enum(a), Typ::Enum(b)) => a == b,
            (Typ::Function(a), Typ::Function(b)) => a == b,
            (Typ::Void, Typ::Void) => true,
            (other, Typ::Dyn) => other != &Typ::Void,
            (Typ::Dyn, other) => other != &Typ::Void,
            _ => false,
        }
    }
}

/// Debug implementation
impl Debug for Typ {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Prelude(prelude) => write!(f, "Type(Prelude({prelude:?}))"),
            Self::Custom(custom) => write!(f, "Type(Custom({}))", custom.borrow().name),
            Self::Enum(custom_enum) => write!(f, "Type(Enum({}))", custom_enum.name),
            Self::Function(function) => write!(f, "Type(Function({}))", function.name),
            Self::Dyn => write!(f, "Type(Dyn)"),
            Self::Void => write!(f, "Type(Void)"),
        }
    }
}

/// T with publicity
#[derive(Debug, Clone, PartialEq)]
pub struct WithPublicity<T: Clone + PartialEq> {
    pub publicity: Publicity,
    pub value: T,
}
