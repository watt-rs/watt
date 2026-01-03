/// Imports
use crate::typ::typ::{Enum, Function, Struct};
use id_arena::{Arena, Id};

/// Type context.
///
/// `TyCx` owns and stores all type-level definitions used by the compiler,
/// such as functions, structs, and enums.
///
/// All definitions are allocated in arenas and are referenced indirectly
/// via typed IDs (`Id<T>`). This provides:
///
/// - zero-cost copying of references
/// - stable identities for types
/// - support for recursive and cyclic type graphs
/// - clear separation between type *references* and type *definitions*
///
/// `TyCx` is expected to live for the entire duration of type checking
/// and later compilation phases.
///
#[derive(Default)]
pub struct TyCx {
    /// Arena storing all function type definitions.
    pub funcs: Arena<Function>,

    /// Arena storing all struct definitions.
    pub structs: Arena<Struct>,

    /// Arena storing all enum definitions.
    pub enums: Arena<Enum>,
}

impl TyCx {
    /// Allocates a new function definition in the type context
    /// and returns its unique ID.
    #[inline]
    pub fn insert_function(&mut self, function: Function) -> Id<Function> {
        self.funcs.alloc(function)
    }

    /// Allocates a new struct definition in the type context
    /// and returns its unique ID.
    #[inline]
    pub fn insert_struct(&mut self, struct_: Struct) -> Id<Struct> {
        self.structs.alloc(struct_)
    }

    /// Allocates a new enum definition in the type context
    /// and returns its unique ID.
    #[inline]
    pub fn insert_enum(&mut self, enum_: Enum) -> Id<Enum> {
        self.enums.alloc(enum_)
    }

    /// Returns an immutable reference to a function definition.
    ///
    /// # Panics
    ///
    /// Panics if the given `id` does not belong to this `TyCx`.
    #[inline]
    pub fn function(&self, id: Id<Function>) -> &Function {
        self.funcs.get(id).expect("invalid Function id")
    }

    /// Returns an immutable reference to a struct definition.
    ///
    /// # Panics
    ///
    /// Panics if the given `id` does not belong to this `TyCx`.
    #[inline]
    pub fn struct_(&self, id: Id<Struct>) -> &Struct {
        self.structs.get(id).expect("invalid Struct id")
    }

    /// Returns an immutable reference to an enum definition.
    ///
    /// # Panics
    ///
    /// Panics if the given `id` does not belong to this `TyCx`.
    #[inline]
    pub fn enum_(&self, id: Id<Enum>) -> &Enum {
        self.enums.get(id).expect("invalid Enum id")
    }

    /// Returns a mutable reference to a function definition.
    ///
    /// # Panics
    ///
    /// Panics if the given `id` does not belong to this `TyCx`.
    #[inline]
    pub fn function_mut(&mut self, id: Id<Function>) -> &mut Function {
        self.funcs.get_mut(id).expect("invalid Function id")
    }

    /// Returns a mutable reference to a struct definition.
    ///
    /// # Panics
    ///
    /// Panics if the given `id` does not belong to this `TyCx`.
    #[inline]
    pub fn struct_mut(&mut self, id: Id<Struct>) -> &mut Struct {
        self.structs.get_mut(id).expect("invalid Struct id")
    }

    /// Returns a mutable reference to an enum definition.
    ///
    /// # Panics
    ///
    /// Panics if the given `id` does not belong to this `TyCx`.
    #[inline]
    pub fn enum_mut(&mut self, id: Id<Enum>) -> &mut Enum {
        self.enums.get_mut(id).expect("invalid Enum id")
    }

    /// Executes `f` with an immutable reference to the function definition
    /// if the given `id` exists in this context.
    ///
    /// Returns `Some(result)` if the function was found, or `None` otherwise.
    #[inline]
    pub fn with_function<R>(&self, id: Id<Function>, f: impl FnOnce(&Function) -> R) -> Option<R> {
        self.funcs.get(id).map(f)
    }

    /// Executes `f` with an immutable reference to the struct definition
    /// if the given `id` exists in this context.
    #[inline]
    pub fn with_struct<R>(&self, id: Id<Struct>, f: impl FnOnce(&Struct) -> R) -> Option<R> {
        self.structs.get(id).map(f)
    }

    /// Executes `f` with an immutable reference to the enum definition
    /// if the given `id` exists in this context.
    #[inline]
    pub fn with_enum<R>(&self, id: Id<Enum>, f: impl FnOnce(&Enum) -> R) -> Option<R> {
        self.enums.get(id).map(f)
    }
}
