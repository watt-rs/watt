/// Imports
use crate::{
    pretty::Pretty,
    typ::{cx::InferCx, def::ModuleDef},
};
use ecow::EcoString;
use id_arena::Id;
use indexmap::IndexMap;
use miette::NamedSource;
use std::{collections::HashMap, fmt::Debug, sync::Arc};
use watt_ast::ast::Publicity;
use watt_common::address::Address;

/// Represents built-in or prelude types in the language.
///
/// `PreludeType` is an enum of primitive types that are
/// available by default in the language, typically without needing
/// an explicit import. These types form the foundation for
/// type checking, expression evaluation, and variable declarations.
///
/// # Variants
///
/// - `Int`
///
///   Represents a signed integer (int64) type. Typically used for whole numbers.
///   Examples: `42`, `-7`.
///
/// - `Float`
///
///   Represents a floating-point (float64) number type. Used for decimal numbers
///   or numbers requiring fractional precision. Examples: `3.14`, `-0.001`.
///
/// - `Bool`
///
///   Represents a boolean type, which can have one of two values:
///   `true` or `false`. Used for logical expressions and control flow.
///
/// - `String`
///
///   Represents a sequence of characters. Used for textual data.
///   Examples: `"hello"`, `"Rust"`.
///
#[derive(Debug, Clone, PartialEq)]
pub enum PreludeType {
    Int,
    Float,
    Bool,
    String,
}

/// Represents a function or enum variant parameter in the language.
///
/// A `Parameter` stores the information about a single parameter
/// of a function or enum, including its type and its location from the
/// source code file.
///
/// # Fields
///
/// - `location: Address`
///   The source code location binding
///
/// - `name: EcoString`
///   The identifier of the parameter
///
/// - `typ: Typ`
///   The type of the parameter. Determines what kind of values
///   can be passed to the function for this parameter. This is
///   used during type checking to ensure correctness.
///
#[derive(Clone, PartialEq)]
pub struct Parameter {
    pub location: Address,
    pub name: EcoString,
    pub typ: Typ,
}

/// Represents a generic parameter in a type or function.
///
/// A `GenericParameter` stores the name and source location of a
/// generic type variable. Used in generic structs, enums or functions
/// to allow type abstraction.
///
/// # Fields
///
/// - `name: EcoString`
///   The identifier of the generic parameter, e.g., `T`, `U`.
///
/// - `typ: usize`
///   Generic ID
///
#[derive(Clone, PartialEq)]
pub struct GenericParameter {
    pub name: EcoString,
    pub id: usize,
}

/// Debug implementation
impl Debug for GenericParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.id)
    }
}

/// Represents a field of a struct.
///
/// A `Field` stores the name, type, and source location of a field
/// in a struct. Used for type checking.
///
/// # Fields
///
/// - `name: EcoString`
///   The identifier of the field.
///
/// - `location: Address`
///   The location in the source code where this field is declared.
///   Includes fields type annotation span too.
///
/// - `typ: Typ`
///   The type  of the field
///
#[derive(Clone, PartialEq)]
pub struct Field {
    pub name: EcoString,
    pub location: Address,
    pub typ: Typ,
}

/// Debug implementation
impl Debug for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.name, self.typ)
    }
}

/// Represents a user-defined structure.
///
/// A `Struct` stores all information about a user-defined struct,
/// including its name, generic parameters, fields, and source location. Used
/// for type checking, generic instantiation and infrenece.
///
/// # Fields
///
/// - `location: Address`
///   The location in the source code where the struct is declared.
///
/// - `uid: usize`
///   A unique identifier for the struct, used internally for type resolution.
///
/// - `name: EcoString`
///   The name of the struct.
///
/// - `generics: Vec<GenericParameter>`
///   A list of generic parameters for the struct.
///
/// - `fields: Vec<Field>`
///   A list of fields in the struct, each with its name, type, and location.
///
#[derive(Clone)]
pub struct Struct {
    pub location: Address,
    pub uid: usize,
    pub name: EcoString,
    pub generics: Vec<GenericParameter>,
    pub fields: Vec<Field>,
}

/// Debug implementation
impl Debug for Struct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Struct({})", self.name)
    }
}

/// PartialEq implementation
///
/// Checks equality of the `Struct` by
/// its unique identifier, that's given
/// during early type checking phase.
///
impl PartialEq for Struct {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}

/// Represents a variant of an enum.
///
/// An `EnumVariant` stores the name, source location, and optional
/// parameters (fields) associated with this variant. Used for
/// type checking, pattern matching, and runtime representation.
///
/// # Fields
///
/// - `location: Address`
///   The location in the source code where this variant is declared.
///
/// - `name: EcoString`
///   The identifier of the variant, e.g., `Some`, `None`.
///
/// - `fields: Vec<Field>`
///   Optional named parameters (fields) for the variant.
///
#[derive(Clone, PartialEq)]
pub struct EnumVariant {
    pub location: Address,
    pub name: EcoString,
    pub fields: Vec<Field>,
}

/// Debug implementation for `EnumVariant`
impl Debug for EnumVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Variant({})", self.name)
    }
}

/// Represents a custom enum type.
///
/// An `Enum` stores all information about a user-defined enum,
/// including its name, generics, variants, and source location.
/// Used for type checking, generic instantiation, pattern matching
/// exhaustiveness check.
///
/// # Fields
///
/// - `location: Address`
///   The location in the source code where the enum is declared.
///
/// - `uid: usize`
///   A unique identifier for the enum, used internally for type resolution
///   and equality checks.
///
/// - `name: EcoString`
///   The name of the enum.
///
/// - `generics: Vec<GenericParameter>`
///   A list of generic parameters for the enum.
///
/// - `variants: Vec<EnumVariant>`
///   A list of variants for this enum, each with its own name, location,
///   and optional parameters.
///
#[derive(Clone)]
#[allow(dead_code)]
pub struct Enum {
    pub location: Address,
    pub uid: usize,
    pub name: EcoString,
    pub generics: Vec<GenericParameter>,
    pub variants: Vec<EnumVariant>,
}

/// Debug implementation for `Enum`
///
/// Displays the enum name in the format `Enum(name)`.
impl Debug for Enum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Enum({})", self.name)
    }
}

/// PartialEq implementation for `Enum`
///
/// Two enums are considered equal if their unique identifiers (`uid`) match.
impl PartialEq for Enum {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}

/// Represents a user-defined function in the language.
///
/// A `Function` stores all necessary information about a function,
/// including its name, parameters, return type, generics, and source location.
///
/// # Fields
///
/// - `location: Address`
///   The location in the source code where the function is declared. Useful
///   for error reporting and debugging.
///
/// - `name: EcoString`
///   The name of the function.
///
/// - `generics: Vec<GenericParameter>`
///   A list of generic parameters for the function, if any.
///
/// - `params: Vec<Parameter>`
///   A list of function parameters, each with a type and source location.
///
/// - `ret: Typ`
///   The inferred return type of the function.
///
#[derive(Clone)]
pub struct Function {
    pub location: Address,
    pub name: EcoString,
    pub generics: Vec<GenericParameter>,
    pub params: Vec<Parameter>,
    pub ret: Typ,
}

/// Debug implementation for `Function`
impl Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Function({})", self.name)
    }
}

/// PartialEq implementation for `Function`
///
/// The `PartialEq` implementation ignores `name` and `location`, and considers
/// two functions equal if they have the same parameters and return type. This
/// is useful for type checking and generic instantiation.
///
impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.params == other.params && self.ret == other.ret
    }
}

/// Represents a module in the language.
///
/// A `Module` contains named definitions (`ModDef`) such as functions,
/// structs, enums, or submodules. Modules serve as namespaces and
/// are used to organize code.
///
/// # Fields
///
/// - `source: Arc<NamedSource<String>>`
///   The source code file where the module is defined.
///
/// - `name: EcoString`
///   The name of the module.
///
/// - `fields: HashMap<EcoString, ModDuleef>`
///   The definitions contained in the module, keyed by their names.
///
#[derive(Clone)]
#[allow(dead_code)]
pub struct Module {
    pub source: Arc<NamedSource<String>>,
    pub name: EcoString,
    pub fields: HashMap<EcoString, ModuleDef>,
}

/// Debug implementation for `Module`
impl Debug for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Module({})", self.name)
    }
}

/// Represents a generic arguments
/// substitutions
#[derive(Clone, PartialEq, Default)]
pub struct GenericArgs {
    /// Substitutions of generics,
    /// `Generic(id)` -> `Typ`
    pub subtitutions: IndexMap<usize, Typ>,
}

/// Debug implementation
impl Debug for GenericArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.subtitutions)
    }
}

/// Represents an inference type variable used during type checking
/// and type inference.
///
/// An `TyVar` can either be unbound (not yet determined) or bound
/// (linked to a concrete type `Typ`).
///
pub enum TyVar {
    /// A type variable that has not yet been resolved.
    /// During inference, this may later be linked to a concrete type.
    Unbound,

    /// A type variable that has already been resolved (substituted)
    /// with a concrete type `Typ`.
    Bound(Typ),
}

/// Represents a type in the language.
///
/// `Typ` covers all possible types, including:
/// - prelude primitive types (`Int`, `Float`, etc.)
/// - user-defined structs and enums
/// - function types
/// - unit type `()`
/// - unbound types for type inference
/// - generic type variables
///
#[derive(Clone, PartialEq, Debug)]
pub enum Typ {
    /// Prelude primitive types
    Prelude(PreludeType),
    /// User-defined struct type with substitutions
    Struct(Id<Struct>, GenericArgs),
    /// User-defined enum type with substitutions
    Enum(Id<Enum>, GenericArgs),
    /// Function type
    Function(Id<Function>, GenericArgs),
    /// Inference type with unique id used during type inference.
    /// (id is used to link unbound `Typ` with substitution)
    Var(Id<TyVar>),
    /// Generic type variable with unique id used during type inference.
    /// (will be replaced with unbounds, during type instantiation)
    Generic(usize),
    /// Unit type, representing `()`
    Unit,
}

/// `Typ` methods implementation
impl Typ {
    /// Retrieves fields and applies
    /// substitution by `InferCx`.
    ///
    /// # Notes
    /// If `Typ` isn't `Typ::Struct(_, _)`, will
    /// return empty vector.
    pub fn fields(&self, icx: &mut InferCx) -> Vec<Field> {
        match self {
            Typ::Struct(id, generics) => icx
                .tcx
                .struct_(*id)
                .fields
                .clone()
                .into_iter()
                .map(|field| Field {
                    location: field.location,
                    name: field.name,
                    typ: icx.mk_fresh_m(field.typ, generics.subtitutions.clone()),
                })
                .collect(),

            _ => vec![],
        }
    }

    /// Retrieves variants and applies
    /// substitution by `InferCx`.
    ///
    /// # Notes
    /// If `Typ` isn't `Typ::Enum(_, _)`, will
    /// return empty vector.
    pub fn variants(&self, icx: &mut InferCx) -> Vec<EnumVariant> {
        // Matching self
        match self {
            Typ::Enum(id, generics) => icx
                .tcx
                .enum_(*id)
                .variants
                .clone()
                .into_iter()
                .map(|variant| EnumVariant {
                    location: variant.location,
                    name: variant.name,
                    fields: variant
                        .fields
                        .into_iter()
                        .map(|field| Field {
                            location: field.location,
                            name: field.name,
                            typ: icx.mk_fresh_m(field.typ, generics.subtitutions.clone()),
                        })
                        .collect(),
                })
                .collect(),
            _ => vec![],
        }
    }

    /// Retrieves params and applies
    /// substitution by `InferCx`.
    ///
    /// # Notes
    /// If `Typ` isn't `Typ::Function(_, _)`, will
    /// return empty vector.
    ///
    pub fn params(&self, icx: &mut InferCx) -> Vec<Parameter> {
        // Matching self
        match self {
            Typ::Function(id, generics) => icx
                .tcx
                .function(*id)
                .params
                .clone()
                .into_iter()
                .map(|param| Parameter {
                    location: param.location.clone(),
                    name: param.name.clone(),
                    typ: icx.mk_fresh_m(param.typ, generics.subtitutions.clone()),
                })
                .collect(),

            _ => vec![],
        }
    }

    /// Retrieves return type and applies
    /// substitution by `InferCx`.
    ///
    /// # Notes
    /// If `Typ` isn't `Typ::Function(_, _)`, will
    /// return Typ::Unit
    ///
    pub fn ret(&self, icx: &mut InferCx) -> Typ {
        // Matching self
        match self {
            Typ::Function(id, generics) => icx.mk_fresh_m(
                icx.tcx.function(*id).ret.clone(),
                generics.subtitutions.clone(),
            ),
            _ => Typ::Unit,
        }
    }
}

/// Pretty printing implementation
impl Pretty for Typ {
    /// Pretty prints type
    ///
    /// # Parameters
    /// - `icx: &mut InferCx`
    ///   Inference context used
    ///   to get struct, enum or function info.
    ///
    fn pretty(&self, icx: &mut InferCx) -> String {
        // Matching self
        match icx.apply(self.clone()) {
            Typ::Prelude(ty) => format!("{ty:?}"),
            Typ::Struct(id, generic_args) if generic_args.subtitutions.len() > 0 => {
                format!(
                    "{}[{}]",
                    icx.tcx.struct_(id).name.clone(),
                    generic_args
                        .subtitutions
                        .values()
                        .cloned()
                        .map(|t| t.pretty(icx))
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
            Typ::Struct(id, _) => {
                format!("{}", icx.tcx.struct_(id).name.clone())
            }
            Typ::Enum(id, generic_args) if generic_args.subtitutions.len() > 0 => {
                format!(
                    "{}[{}]",
                    icx.tcx.enum_(id).name.clone(),
                    generic_args
                        .subtitutions
                        .values()
                        .cloned()
                        .map(|t| t.pretty(icx))
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
            Typ::Enum(id, _) => {
                format!("{}", icx.tcx.enum_(id).name.clone())
            }
            it @ Typ::Function(_, _) => {
                format!(
                    "({}) -> {}",
                    it.params(icx)
                        .iter()
                        .map(|p| p.typ.pretty(icx))
                        .collect::<Vec<String>>()
                        .join(", "),
                    it.ret(icx).pretty(icx)
                )
            }
            Typ::Var(id) => format!("?{}", id.index()),
            Typ::Generic(id) => format!("^{id}"),
            Typ::Unit => format!("Unit"),
        }
    }
}

/// Wraps a value with its publicity information.
///
/// `WithPublicity` is a generic struct used to attach access
/// modifiers (like `public` or `private`) to any value.
///
/// # Fields
///
/// - `publicity: Publicity`
///   The visibility modifier of the value.
///
/// - `value: T`
///   The value being wrapped.
///
#[derive(Clone, PartialEq)]
pub struct WithPublicity<T: Clone + PartialEq> {
    pub publicity: Publicity,
    pub value: T,
}

/// Debug implementation for `WithPublicity<T>`
///
/// Displays the value along with its publicity modifier for debugging.
impl<T: Debug + Clone + PartialEq> Debug for WithPublicity<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WithPublicity({:?}, {:?})", self.publicity, self.value)
    }
}
