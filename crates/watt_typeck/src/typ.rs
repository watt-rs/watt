/// Imports
use crate::resolve::resolve::ModDef;
use ecow::EcoString;
use miette::NamedSource;
use std::{cell::RefCell, collections::HashMap, fmt::Debug, sync::Arc};
use watt_ast::ast::Publicity;
use watt_common::{address::Address, rc_ptr::RcPtr};

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
#[allow(dead_code)]
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
#[derive(Clone, PartialEq)]
pub struct EnumVariant {
    pub location: Address,
    pub name: EcoString,
    pub params: HashMap<EcoString, Typ>,
}

/// Debug implementation
impl Debug for EnumVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Variant({})", self.name)
    }
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

/// PartialEq implementation
/// ignores function name and location
impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source && self.params == other.params && self.ret == other.ret
    }
}

/// Custom type
#[derive(Clone, PartialEq)]
pub enum CustomType {
    Enum(RcPtr<Enum>),
    Type(RcPtr<RefCell<Type>>),
}

/// Debug implementation
impl Debug for CustomType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CustomType::Enum(en) => write!(f, "Custom({en:?})"),
            CustomType::Type(ty) => write!(f, "Custom({ty:?})"),
        }
    }
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
    Unit,
}

/// PartialEq implementation
impl PartialEq for Typ {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Typ::Prelude(a), Typ::Prelude(b)) => a == b,
            (Typ::Custom(a), Typ::Custom(b)) => a == b,
            (Typ::Enum(a), Typ::Enum(b)) => a == b,
            (Typ::Function(a), Typ::Function(b)) => a.veq(b),
            (Typ::Unit, Typ::Unit) => true,
            (other, Typ::Dyn) => other != &Typ::Unit,
            (Typ::Dyn, other) => other != &Typ::Unit,
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
            Self::Unit => write!(f, "Type(Unit)"),
        }
    }
}

/// T with publicity
#[derive(Clone, PartialEq)]
pub struct WithPublicity<T: Clone + PartialEq> {
    pub publicity: Publicity,
    pub value: T,
}

/// Debug implementation
impl<T: Debug + Clone + PartialEq> Debug for WithPublicity<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WithPublicity({:?}, {:?})", self.publicity, self.value)
    }
}
