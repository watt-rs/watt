/// Imports
use crate::{
    cx::module::ModuleCx,
    inference::equation::Equation,
    typ::{
        def::ModuleDef,
        res::Res,
        typ::{Function, Parameter, Typ, WithPublicity},
    },
};
use ecow::EcoString;
use indexmap::IndexMap;
use std::rc::Rc;
use watt_ast::ast::{
    self, Block, Either, Expression, FnDeclaration,
    Publicity, TypePath,
};
use watt_common::address::Address;
use crate::inference::equation::EqUnit;

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
        publicity: Publicity,
        name: EcoString,
        params: Vec<ast::Parameter>,
        ret_type: Option<TypePath>,
        body: Either<Block, Expression>,
    ) {
        // Requesting function
        let f = match self.resolver.resolve(&location, &name) {
            Res::Value(Typ::Function(f)) => f,
            _ => unreachable!(),
        };

        // Pushing generics
        self.solver
            .hydrator
            .generics
            .re_push_scope(f.generics.clone());

        // inferring return type
        let ret = ret_type.map_or(Typ::Unit, |t| self.infer_type_annotation(t));

        // inferred params
        let params = params
            .into_iter()
            .map(|p| {
                (
                    p.name,
                    Parameter {
                        location: p.location,
                        typ: self.infer_type_annotation(p.typ),
                    },
                )
            })
            .collect::<IndexMap<EcoString, Parameter>>();

        // creating and defining function
        let function = Function {
            location: location.clone(),
            name: name.clone(),
            generics: f.generics.clone(),
            params: params.clone().into_values().collect(),
            ret: ret.clone(),
        };
        self.resolver.define_module(
            &location,
            &name,
            ModuleDef::Function(WithPublicity {
                publicity,
                value: Rc::new(function),
            }),
            true,
        );

        // pushing new scope
        self.resolver.push_rib();

        // defining params in new scope
        params.iter().for_each(|p| {
            self.resolver
                .define_local(&location, p.0, p.1.typ.clone(), false)
        });

        // inferring body
        let (block_location, inferred_block) = match body {
            Either::Left(block) => (block.location.clone(), self.infer_block(block)),
            Either::Right(expr) => (expr.location(), self.infer_expr(expr)),
        };
        self.solver.solve(Equation::Unify(
            EqUnit(location, ret),
            EqUnit(block_location, inferred_block),
        ));
        self.resolver.pop_rib();

        // Popping generics
        self.solver.hydrator.generics.pop_scope();
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
            publicity,
            name,
            params,
            typ,
            body,
            ..
        } = decl { self.late_analyze_fn(location, publicity, name, params, typ, body) }
    }
}
