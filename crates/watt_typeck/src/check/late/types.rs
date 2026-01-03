/// Imports
use crate::{
    cx::module::ModuleCx,
    typ::{
        def::TypeDef,
        typ::{Enum, EnumVariant, Field, Struct},
    },
};
use ecow::EcoString;
use watt_ast::ast::{self, EnumConstructor, TypeDeclaration};
use watt_common::address::Address;

/// Late declaration analysis pass for the module.
///
/// This pass completes the semantic analysis of declarations (structs, enums,
/// functions, constants) after their names and initial shells
/// have been registered during the early phase.
///
/// In this stage:
/// - Generic parameters are reinstated into the inference context.
/// - All type annotations are resolved into `Typ`.
/// - Function bodies are type-checked.
/// - Struct and enum fields are fully typed.
/// - Constants are inferred and unified against their annotations.
/// - All definitions are finalized and registered into the resolver.
///
impl<'pkg, 'cx> ModuleCx<'pkg, 'cx> {
    /// Performs late analysis of a struct declaration.
    ///
    /// ## Responsibilities:
    /// - Re-push the struct's generic parameters into the type hydrator.
    /// - Infer the types of all fields using `infer_type_annotation`.
    /// - Rebuild the `Struct` def with resolved field types.
    /// - Overwrite the existing struct definition with the completed one.
    ///
    /// This operation mutates the struct in place, finalizing its type
    /// structure for the rest of type checking.
    ///
    fn late_analyze_struct(&mut self, location: Address, name: EcoString, fields: Vec<ast::Field>) {
        // Requesting struct
        let id = match self.resolver.resolve_type(&location, &name) {
            TypeDef::Struct(ty) => ty,
            _ => unreachable!(),
        };
        let struct_ = self.tcx.struct_(id);
        let (location, uid, name, generics) = (
            struct_.location.clone(),
            struct_.uid,
            struct_.name.clone(),
            struct_.generics.clone(),
        );

        // Re pushing generics
        self.solver
            .hydrator
            .generics
            .re_push_scope(struct_.generics.clone());

        // Inferencing fields
        let new_struct = Struct {
            location: location.clone(),
            uid: uid,
            name: name.clone(),
            generics: generics.clone(),
            fields: fields
                .into_iter()
                .map(|f| Field {
                    name: f.name,
                    location: f.location,
                    typ: self.infer_type_annotation(f.typ),
                })
                .collect(),
        };
        let struct_mut = self.tcx.struct_mut(id);
        *struct_mut = new_struct;

        // Popping generics
        self.solver.hydrator.generics.pop_scope();
    }

    /// Performs late analysis of an enum declaration.
    ///
    /// ## Responsibilities:
    /// - Re-push the enum’s generic parameters in the hydrator.
    /// - Infer the types of all variant fields.
    /// - Rebuild the `Enum` def with resolved variant field types.
    /// - Overwrite the existing enum definition with the completed one.
    ///
    /// Enum variant fields are treated similarly to struct fields: each
    /// parameter is analyzed using `infer_type_annotation`.
    ///
    fn late_analyze_enum(
        &mut self,
        location: Address,
        name: EcoString,
        variants: Vec<EnumConstructor>,
    ) {
        // Requesting enum
        let id = match self.resolver.resolve_type(&location, &name) {
            TypeDef::Enum(en) => en,
            _ => unreachable!(),
        };
        let enum_ = self.tcx.enum_(id);
        let (location, uid, name, generics) = (
            enum_.location.clone(),
            enum_.uid,
            enum_.name.clone(),
            enum_.generics.clone(),
        );
        
        // Repushing generics
        self.solver
            .hydrator
            .generics
            .re_push_scope(generics.clone());

        // Inferencing fields
        let new_enum = Enum {
            location: location.clone(),
            uid: uid,
            name: name.clone(),
            generics: generics.clone(),
            variants: variants
                .into_iter()
                .map(|v| EnumVariant {
                    location: v.location,
                    name: v.name,
                    fields: v
                        .params
                        .into_iter()
                        .map(|p: ast::Parameter| Field {
                            location: p.location,
                            name: p.name,
                            typ: self.infer_type_annotation(p.typ),
                        })
                        .collect(),
                })
                .collect(),
        };
        let enum_mut = self.tcx.enum_mut(id);
        *enum_mut = new_enum;

        // Popping generics
        self.solver.hydrator.generics.pop_scope();
    }

    /// Dispatches a type declaration to the corresponding late analysis routine.
    ///
    /// Each type declaration variant is fully processed here:
    /// - Struct → `late_analyze_struct`
    /// - Enum → `late_analyze_enum`
    ///
    /// After this call, each type declaration is fully type-analyzed and integrated
    /// into the module’s type environment.
    ///
    pub fn late_analyze_type_decl(&mut self, decl: TypeDeclaration) {
        match decl {
            TypeDeclaration::Struct {
                location,
                name,
                fields,
                ..
            } => self.late_analyze_struct(location, name, fields),
            TypeDeclaration::Enum {
                location,
                name,
                variants,
                ..
            } => self.late_analyze_enum(location, name, variants),
        }
    }
}
