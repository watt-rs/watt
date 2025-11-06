/// Imports
use crate::resolve::resolve::ModDef;
use ecow::EcoString;
use miette::NamedSource;
use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc, sync::Arc};
use watt_ast::ast::Publicity;
use watt_common::address::Address;

/// Represetns prelude
/// (or build-in type)
#[derive(Debug, Clone, PartialEq)]
pub enum PreludeType {
    Int,
    Float,
    Bool,
    String,
}

/// Parameter
#[derive(Clone, PartialEq)]
pub struct Parameter {
    pub location: Address,
    pub typ: Typ,
}

/// Custom type
#[derive(Clone)]
pub struct Struct {
    pub source: Arc<NamedSource<String>>,
    pub location: Address,
    pub uid: usize,
    pub name: EcoString,
    pub params: Vec<Parameter>,
    pub env: HashMap<EcoString, WithPublicity<Typ>>,
}

/// Debug implementation
impl Debug for Struct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Type({})", self.name)
    }
}

/// PartialEq implementation
impl PartialEq for Struct {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}

/// Implementation
impl Struct {
    /// Checks that type implements trait
    pub fn is_impls(&self, tr: Rc<Trait>) -> bool {
        // Checking functions matches trait functions
        for function in tr.functions.values() {
            // Checking that function exists in type
            match self.env.get(&function.name) {
                Some(def) => {
                    match &def.value {
                        Typ::Function(implementation) => {
                            // If function == implementation
                            if function == implementation {
                                // Checking implementation publicity
                                if def.publicity != Publicity::Public {
                                    return false;
                                }
                            } else {
                                return false;
                            }
                        }
                        _ => return false,
                    }
                }
                None => return false,
            }
        }
        // If all is implemented => true
        true
    }
}

/// Trait
#[derive(Clone)]
pub struct Trait {
    pub source: Arc<NamedSource<String>>,
    pub location: Address,
    pub uid: usize,
    pub name: EcoString,
    pub functions: HashMap<EcoString, Rc<Function>>,
}

/// Debug implementation
impl Debug for Trait {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Trait({})", self.name)
    }
}

/// PartialEq implementation
impl PartialEq for Trait {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
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
    pub source: Arc<NamedSource<String>>,
    pub location: Address,
    pub uid: usize,
    pub name: EcoString,
    pub variants: Vec<EnumVariant>,
}

/// Debug implementation
impl Debug for Enum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Enum({})", self.name)
    }
}

/// PartialEq implementation
impl PartialEq for Enum {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}

/// Function
#[derive(Clone)]
pub struct Function {
    pub source: Arc<NamedSource<String>>,
    pub location: Address,
    pub uid: usize,
    pub name: EcoString,
    pub params: Vec<Parameter>,
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
        self.params == other.params && self.ret == other.ret
    }
}

/// Custom type
#[derive(Clone, PartialEq)]
pub enum CustomType {
    Enum(Rc<Enum>),
    Struct(Rc<RefCell<Struct>>),
    Trait(Rc<Trait>),
}

/// Debug implementation
impl Debug for CustomType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CustomType::Enum(en) => write!(f, "Custom({en:?})"),
            CustomType::Struct(ty) => write!(f, "Custom({ty:?})"),
            CustomType::Trait(tr) => write!(f, "Custom({tr:?})"),
        }
    }
}

/// Module
#[derive(Clone)]
#[allow(dead_code)]
pub struct Module {
    pub source: Arc<NamedSource<String>>,
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
    Struct(Rc<RefCell<Struct>>),
    Trait(Rc<Trait>),
    Enum(Rc<Enum>),
    Function(Rc<Function>),
    Dyn,
    Unit,
}

/// PartialEq implementation
impl PartialEq for Typ {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Typ::Prelude(a), Typ::Prelude(b)) => a == b,
            (Typ::Struct(a), Typ::Struct(b)) => a == b,
            (Typ::Enum(a), Typ::Enum(b)) => a == b,
            (Typ::Trait(a), Typ::Trait(b)) => a == b,
            (Typ::Function(a), Typ::Function(b)) => a == b,
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
            Self::Struct(custom) => write!(f, "Type(Struct({}))", custom.borrow().name),
            Self::Enum(custom_enum) => write!(f, "Type(Enum({}))", custom_enum.name),
            Self::Function(function) => write!(f, "Type(Function({}))", function.name),
            Self::Dyn => write!(f, "Type(Dyn)"),
            Self::Unit => write!(f, "Type(Unit)"),
            Self::Trait(custom_trait) => write!(f, "Type(Trait({}))", custom_trait.name),
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
