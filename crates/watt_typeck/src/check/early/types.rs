/// Imports
use crate::cx::module::ModuleCx;
use crate::typ::def::{ModuleDef, TypeDef};
use crate::typ::typ::{Enum, Struct, WithPublicity};
use ecow::EcoString;
use watt_ast::ast::{Publicity, TypeDeclaration};
use watt_common::address::Address;

/// Implementation of the type declarations early analyse.
///
/// Enums and structs are defined *just by name* without any
/// semantic analyse pass. Enums and structs semantic pass
/// will be performed in the last phase.
///
impl<'pkg, 'cx> ModuleCx<'pkg, 'cx> {
    /// Registers a struct name in the module before its fields are analyzed.
    ///
    /// This creates a placeholder [`Struct`] with:
    /// - empty fields,
    /// - generic params,
    /// - fresh `uid`,
    ///   but without performing any semantic checks.
    ///
    /// The full struct body will later be populated in
    /// [`late_analyze_struct`].
    ///
    pub(crate) fn early_define_struct(
        &mut self,
        location: Address,
        publicity: Publicity,
        generics: Vec<EcoString>,
        name: EcoString,
    ) {
        // Pushing generics
        let generics = self.icx.generics.push_scope(generics);
        // Generating struct
        let struct_ = Struct {
            location: location.clone(),
            uid: self.fresh_id(),
            name: name.clone(),
            generics,
            fields: Vec::new(),
        };
        let id = self.icx.tcx.insert_struct(struct_);
        // Popping generics
        self.icx.generics.pop_scope();
        // Defining struct
        self.resolver.define_module(
            &location,
            &name,
            ModuleDef::Type(WithPublicity {
                publicity,
                value: TypeDef::Struct(id),
            }),
            false,
        );
    }

    /// Registers an enum name in the module before its variants are analyzed.
    ///
    /// The enum is inserted as a placeholder containing:
    /// - generics params,
    /// - no variants,
    /// - fresh `uid`.
    ///
    /// Variants and their fields are added later during
    /// [`late_analyze_enum`].
    ///
    pub(crate) fn early_define_enum(
        &mut self,
        location: Address,
        publicity: Publicity,
        generics: Vec<EcoString>,
        name: EcoString,
    ) {
        // Pushing generics
        let generics = self.icx.generics.push_scope(generics);
        // Generating enum
        let enum_ = Enum {
            location: location.clone(),
            uid: self.fresh_id(),
            name: name.clone(),
            generics,
            variants: Vec::new(),
        };
        let id = self.icx.tcx.insert_enum(enum_);
        // Popping generics
        self.icx.generics.pop_scope();
        // Defining enum
        self.resolver.define_module(
            &location,
            &name,
            ModuleDef::Type(WithPublicity {
                publicity,
                value: TypeDef::Enum(id),
            }),
            false,
        );
    }

    /// Dispatches early-phase definition for any kind of type declaration.
    ///
    /// Each declaration type is handled by the corresponding `early_define_*`
    /// method. This ensures all top-level symbols are registered before any
    /// “late” semantic analysis runs.
    ///
    pub(crate) fn early_analyze_type_decl(&mut self, declaration: &TypeDeclaration) {
        // Matching declaration
        match declaration.clone() {
            TypeDeclaration::Struct {
                location,
                name,
                publicity,
                generics,
                ..
            } => self.early_define_struct(location, publicity, generics, name),
            TypeDeclaration::Enum {
                location,
                name,
                publicity,
                generics,
                ..
            } => self.early_define_enum(location, publicity, generics, name),
        }
    }
}
