/// Modules
mod functions;
mod types;

/// Imports
use crate::{
    cx::module::ModuleCx,
    errors::TypeckError,
    inference::coercion::Coercion,
    typ::{def::ModuleDef, typ::WithPublicity},
};
use ecow::EcoString;
use watt_ast::ast::{Declaration, Dependency, Expression, Publicity, TypePath, UseKind};
use watt_common::{address::Address, bail};

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
    /// Performs late analysis of a constant definition.
    ///
    /// A constant has:
    /// - A required type annotation.
    /// - A value expression which is inferred independently.
    ///
    /// ## Procedure:
    /// 1. Resolve the type annotation.
    /// 2. Infer the type of the value expression.
    /// 3. Emit a unification constraint requiring the expression type to match
    ///    the annotated type.
    /// 4. Register the constant in the module namespace.
    ///
    /// ## Constants do not:
    /// - Introduce generics.
    /// - Create scopes.
    /// - Participate in inference outside their own value.
    ///
    fn late_define_const(
        &mut self,
        location: Address,
        publicity: Publicity,
        name: EcoString,
        typ: TypePath,
        value: Expression,
    ) {
        // Const inference
        let annotated_location = typ.location();
        let annotated = self.infer_type_annotation(typ);
        let inferred_location = value.location();
        let inferred = self.infer_expr(value);
        self.solver.coerce(
            self.tcx,
            Coercion::Eq(
                (annotated_location, annotated.clone()),
                (inferred_location, inferred),
            ),
        );

        // Defining constant
        self.resolver.define_module(
            &location,
            &name,
            ModuleDef::Const(WithPublicity {
                publicity,
                value: annotated,
            }),
            false,
        );
    }

    /// Each declaration variant is fully processed here:
    /// - Const → `late_analyze_const`
    /// - Type → `late_analyze_type`
    /// - Fn → `late_analyze_fn`
    ///
    /// After this call, each declaration is fully type-analyzed and integrated
    /// into the module’s type environment.
    ///
    pub fn late_analyze_decl(&mut self, declaration: Declaration) {
        match declaration {
            Declaration::Type(decl) => self.late_analyze_type_decl(decl),
            Declaration::Fn(decl) => self.late_analyze_fn_decl(decl),
            Declaration::Const(decl) => self.late_define_const(
                decl.location,
                decl.publicity,
                decl.name,
                decl.typ,
                decl.value,
            ),
        }
    }

    /// Performs an import of another module into the current resolver scope.
    ///
    /// ## Supports:
    /// - `use foo as name`  → import with renamed binding.
    /// - `use foo for a,b`  → import selected names.
    ///
    /// On success, the referenced module is integrated into the current scope
    /// according to the chosen `UseKind`.
    ///
    /// ## Errors
    /// - [`TypeckError::ImportOfUnknownModule`]: if module doesn't exist.
    ///
    pub fn perform_import(&mut self, import: Dependency) {
        match self.package.root.modules.get(&import.path.module) {
            Some(module) => match import.kind {
                UseKind::AsName(name) => {
                    self.resolver
                        .import_as(&import.location, name, module.clone())
                }
                UseKind::ForNames(names) => {
                    self.resolver
                        .import_for(&import.location, names, module.clone())
                }
            },
            None => bail!(TypeckError::ImportOfUnknownModule {
                src: self.module.source.clone(),
                span: import.location.span.into(),
                m: import.path.module
            }),
        };
    }
}
