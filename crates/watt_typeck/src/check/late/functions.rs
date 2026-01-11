/// Imports
use crate::{
    cx::module::ModuleCx,
    inference::{cause::Cause, coercion::{self, Coercion}},
    typ::{
        res::Res,
        typ::{Parameter, Typ},
    },
};
use ecow::EcoString;
use watt_ast::ast::{Block, Either, Expression, FnDeclaration};
use watt_common::address::Address;

/// Late declaration analysis pass for the module.
///
/// This pass completes the semantic analysis of declarations (structs, enums,
/// functions, extern functions, constants) after their names and initial shells
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
    /// Performs late analysis of a user-defined function.
    ///
    /// ## Steps:
    /// - Look up the function shell previously registered by the early pass.
    /// - Re-push generic parameters into the hydrator.
    /// - Resolve the return type if annotated; otherwise assume `Unit`.
    /// - Resolve the types of all parameters, constructing a typed signature.
    /// - Publish the function signature into the module (so it is visible to
    ///    recursive calls within its own body).
    /// - Create a new scope (rib) for local variables.
    /// - Insert parameters as locals into that scope.
    /// - Infer the function body (block or expression).
    /// - Emit a unification equation requiring: `inferred_body_type == return_type`.
    /// - Pop the local scope.
    /// - Pop the generic parameter scope.
    ///
    /// At the end of this method the function is fully type-checked.
    fn late_analyze_fn(
        &mut self,
        location: Address,
        name: EcoString,
        body: Either<Block, Expression>,
    ) {
        // Requesting function
        let id = match self.resolver.resolve(&location, &name) {
            Res::Value(Typ::Function(f, _)) => f,
            _ => unreachable!(),
        };
        let function = self.icx.tcx.function_mut(id);
        let params: Vec<Parameter> = function.params.clone();
        let ret = function.ret.clone();
        let generics = function.generics.clone();

        // Pushing generics
        self.icx.generics.re_push_scope(generics.clone());

        // pushing new scope
        self.resolver.push_rib();

        // defining params in new scope
        params.iter().for_each(|p| {
            self.resolver
                .define_local(&location, &p.name, p.typ.clone(), false)
        });

        // inferring body
        let (block_location, inferred_block) = match body {
            Either::Left(block) => (block.location.clone(), self.infer_block(block)),
            Either::Right(expr) => (expr.location(), self.infer_expr(expr)),
        };
        coercion::coerce(
            &mut self.icx,
            Cause::Return(&block_location, &location),
            Coercion::Eq(inferred_block, ret),
        );
        self.resolver.pop_rib();

        // Popping generics
        self.icx.generics.pop_scope();
    }

    /// Dispatches a function declaration to the corresponding late analysis routine.
    ///
    /// Each type declaration variant is fully processed here:
    /// - Function → `late_analyze_function_decl`
    /// - Extern → `skip!()`
    ///
    /// After this call, each function declaration is fully type-analyzed and integrated
    /// into the module’s type environment.
    ///
    /// Dispatches a declaration to the corresponding late analysis routine.
    ///
    /// # Notes
    /// Externals does not need any additional analyze after early.
    ///
    pub fn late_analyze_fn_decl(&mut self, decl: FnDeclaration) {
        if let FnDeclaration::Function {
            location,
            name,
            body,
            ..
        } = decl
        {
            self.late_analyze_fn(location, name, body)
        }
    }
}
