/// Imports
use crate::{
    cx::module::ModuleCx,
    errors::TypeckError,
    inference::equation::Equation,
    typ::{
        def::{ModuleDef, TypeDef},
        res::Res,
        typ::{Enum, EnumVariant, Field, Function, Parameter, Struct, Typ, WithPublicity},
    },
};
use ecow::EcoString;
use indexmap::IndexMap;
use std::rc::Rc;
use watt_ast::ast::{
    self, Block, Declaration, Dependency, Either, EnumConstructor, Expression, FnDeclaration,
    Publicity, TypeDeclaration, TypePath, UseKind,
};
use watt_common::{address::Address, bail};

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
        let ty = match self.resolver.resolve_type(&location, &name) {
            TypeDef::Struct(ty) => ty,
            _ => unreachable!(),
        };
        let borrowed = ty.borrow();

        // Repushing generics
        self.solver
            .hydrator
            .generics
            .re_push_scope(borrowed.generics.clone());

        // Inferencing fields
        let new_struct = Struct {
            location: borrowed.location.clone(),
            uid: borrowed.uid,
            name: borrowed.name.clone(),
            generics: borrowed.generics.clone(),
            fields: fields
                .into_iter()
                .map(|f| Field {
                    name: f.name,
                    location: f.location,
                    typ: self.infer_type_annotation(f.typ),
                })
                .collect(),
        };
        drop(borrowed);
        *ty.borrow_mut() = new_struct;

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
        let en = match self.resolver.resolve_type(&location, &name) {
            TypeDef::Enum(en) => en,
            _ => unreachable!(),
        };
        let borrowed = en.borrow();

        // Repushing generics
        self.solver
            .hydrator
            .generics
            .re_push_scope(borrowed.generics.clone());

        // Inferencing fields
        let new_enum = Enum {
            location: borrowed.location.clone(),
            uid: borrowed.uid,
            name: borrowed.name.clone(),
            generics: borrowed.generics.clone(),
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
        drop(borrowed);
        *en.borrow_mut() = new_enum;

        // Popping generics
        self.solver.hydrator.generics.pop_scope();
    }

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
    fn late_analyze_function_decl(
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
            (location, ret),
            (block_location, inferred_block),
        ));
        self.resolver.pop_rib();

        // Popping generics
        self.solver.hydrator.generics.pop_scope();
    }

    /// Performs late analysis of an extern function.
    ///
    /// ## Unlike normal functions, extern functions:
    /// - Have no body to analyze.
    /// - Only require inference of parameter and return type annotations.
    ///
    /// ## Steps:
    /// - Retrieve the function declaration shell.
    /// - Push generics into the hydrator.
    /// - Resolve return and parameter types.
    /// - Publish the completed function signature.
    /// - Pop the generic scope.
    ///
    fn late_analyze_extern_decl(
        &mut self,
        location: Address,
        publicity: Publicity,
        name: EcoString,
        params: Vec<ast::Parameter>,
        ret_type: Option<TypePath>,
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

        // Popping generics
        self.solver.hydrator.generics.pop_scope();
    }

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
        self.solver.solve(Equation::Unify(
            (annotated_location, annotated.clone()),
            (inferred_location, inferred),
        ));

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

    /// Dispatches a type declaration to the corresponding late analysis routine.
    ///
    /// Each type declaration variant is fully processed here:
    /// - Struct → `late_analyze_struct`
    /// - Enum → `late_analyze_enum`
    ///
    /// After this call, each type declaration is fully type-analyzed and integrated
    /// into the module’s type environment.
    ///
    pub fn late_analyze_type_declaration(&mut self, decl: TypeDeclaration) {
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

    /// Dispatches a function declaration to the corresponding late analysis routine.
    ///
    /// Each type declaration variant is fully processed here:
    /// - Function → `late_analyze_function_decl`
    /// - Extern → `late_analyze_extern_decl`
    ///
    /// After this call, each function declaration is fully type-analyzed and integrated
    /// into the module’s type environment.
    ///
    /// Dispatches a declaration to the corresponding late analysis routine.
    pub fn late_analyze_fn_declaration(&mut self, decl: FnDeclaration) {
        match decl {
            FnDeclaration::Function {
                location,
                publicity,
                name,
                params,
                typ,
                body,
                ..
            } => self.late_analyze_function_decl(location, publicity, name, params, typ, body),
            FnDeclaration::ExternFunction {
                location,
                publicity,
                name,
                params,
                typ,
                ..
            } => self.late_analyze_extern_decl(location, publicity, name, params, typ),
        }
    }

    /// Each declaration variant is fully processed here:
    /// - Const → `late_define_const`
    ///
    /// After this call, each declaration is fully type-analyzed and integrated
    /// into the module’s type environment.
    ///
    pub fn late_analyze_declaration(&mut self, declaration: Declaration) {
        match declaration {
            Declaration::Type(decl) => self.late_analyze_type_declaration(decl),
            Declaration::Fn(decl) => self.late_analyze_fn_declaration(decl),
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
