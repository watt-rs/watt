use crate::inference::equation::EqUnit;
/// Imports
use crate::{
    cx::module::ModuleCx,
    errors::{TypeckError, TypeckRelated},
    ex::ExMatchCx,
    inference::equation::Equation,
    typ::{
        def::{ModuleDef, TypeDef},
        res::Res,
        typ::{Function, Parameter, PreludeType, Typ},
    },
    warnings::TypeckWarning,
};
use ecow::EcoString;
use indexmap::IndexMap;
use std::{collections::HashMap, rc::Rc};
use watt_ast::ast::{
    self, BinaryOp, Block, Case, Either, ElseBranch, Expression, Pattern, Publicity, TypePath,
    UnaryOp,
};
use watt_common::{address::Address, bail, skip, warn};

/// Expressions inferring
impl<'pkg, 'cx> ModuleCx<'pkg, 'cx> {
    /// Infers the type of concat expression.
    ///
    /// This function:
    /// - Checks that both the left and right operands are strings.
    /// - Produces the resulting type, or emits a `TypeckError::InvalidBinaryOp`.
    ///
    ///
    /// # Parameters
    /// - `location`: Source code address of the binary operator.
    /// - `left`: Left-hand side type.
    /// - `right`: Right-hand side type.
    ///
    /// # Returns
    /// -`Typ::String`
    ///
    fn infer_binary_concat(&self, location: Address, left: Typ, right: Typ) -> Typ {
        // Checking prelude types
        match left {
            Typ::Prelude(PreludeType::String) => match right {
                Typ::Prelude(PreludeType::String) => Typ::Prelude(PreludeType::String),
                _ => bail!(TypeckError::InvalidBinaryOp {
                    src: self.module.source.clone(),
                    span: location.span.into(),
                    a: left,
                    b: right,
                    op: BinaryOp::Concat
                }),
            },
            _ => bail!(TypeckError::InvalidBinaryOp {
                src: self.module.source.clone(),
                span: location.span.into(),
                a: left,
                b: right,
                op: BinaryOp::Concat
            }),
        }
    }

    /// Infers the type of arithmetical expression.
    ///
    /// This function:
    /// - Checks that both the left and right operands are numeric.
    /// - Produces the resulting type, or emits a `TypeckError::InvalidBinaryOp`.
    ///
    /// # Parameters
    /// - `location`: Source code address of the binary operator.
    /// - `left`: Left-hand side type.
    /// - `op`: Binary operator used for the diagnostics.
    /// - `right`: Right-hand side type.
    ///
    /// # Returns
    /// - The resulting `Typ` after applying the operator.
    ///
    /// # Notes
    /// Numeric operators automatically promote `Int × Float` or `Float × Int` to `Float`.
    ///
    fn infer_binary_arithmetical(
        &self,
        location: Address,
        left: Typ,
        op: BinaryOp,
        right: Typ,
    ) -> Typ {
        // Checking prelude types
        match left {
            Typ::Prelude(PreludeType::Int) => match right {
                Typ::Prelude(PreludeType::Int) => Typ::Prelude(PreludeType::Int),
                Typ::Prelude(PreludeType::Float) => Typ::Prelude(PreludeType::Float),
                _ => bail!(TypeckError::InvalidBinaryOp {
                    src: self.module.source.clone(),
                    span: location.span.into(),
                    a: left,
                    b: right,
                    op
                }),
            },
            Typ::Prelude(PreludeType::Float) => match right {
                Typ::Prelude(PreludeType::Int) => Typ::Prelude(PreludeType::Float),
                Typ::Prelude(PreludeType::Float) => Typ::Prelude(PreludeType::Float),
                _ => bail!(TypeckError::InvalidBinaryOp {
                    src: self.module.source.clone(),
                    span: location.span.into(),
                    a: left,
                    b: right,
                    op
                }),
            },
            _ => bail!(TypeckError::InvalidBinaryOp {
                src: self.module.source.clone(),
                span: location.span.into(),
                a: left,
                b: right,
                op
            }),
        }
    }

    /// Infers the type of logical expression.
    ///
    /// This function:
    /// - Checks that both the left and right operands are `Typ::Bool`.
    /// - Produces the resulting type, or emits a `TypeckError::InvalidBinaryOp`.
    ///
    /// # Parameters
    /// - `location`: Source code address of the binary operator.
    /// - `left`: Left-hand side type.
    /// - `op`: Binary operator used for the diagnostics.
    /// - `right`: Right-hand side type.
    ///
    /// # Returns
    /// - `Typ::Bool`
    ///
    fn infer_binary_logical(&self, location: Address, left: Typ, op: BinaryOp, right: Typ) -> Typ {
        // Checking prelude types
        match left {
            Typ::Prelude(PreludeType::Bool) => match right {
                Typ::Prelude(PreludeType::Bool) => Typ::Prelude(PreludeType::Bool),
                _ => bail!(TypeckError::InvalidBinaryOp {
                    src: self.module.source.clone(),
                    span: location.span.into(),
                    a: left,
                    b: right,
                    op
                }),
            },
            _ => bail!(TypeckError::InvalidBinaryOp {
                src: self.module.source.clone(),
                span: location.span.into(),
                a: left,
                b: right,
                op
            }),
        }
    }

    /// Infers the type of compare expression.
    ///
    /// This function:
    /// - Checks that both the left and right operands are numerics.
    /// - Produces the resulting type, or emits a `TypeckError::InvalidBinaryOp`.
    ///
    /// # Parameters
    /// - `location`: Source code address of the binary operator.
    /// - `left`: Left-hand side type.
    /// - `op`: Binary operator used for the diagnostics.
    /// - `right`: Right-hand side type.
    ///
    /// # Returns
    /// - `Typ::Bool`
    ///
    fn infer_binary_compare(&self, location: Address, left: Typ, op: BinaryOp, right: Typ) -> Typ {
        // Checking prelude types
        match left {
            Typ::Prelude(PreludeType::Int) | Typ::Prelude(PreludeType::Float) => match right {
                Typ::Prelude(PreludeType::Int) | Typ::Prelude(PreludeType::Float) => {
                    Typ::Prelude(PreludeType::Bool)
                }
                _ => bail!(TypeckError::InvalidBinaryOp {
                    src: self.module.source.clone(),
                    span: location.span.into(),
                    a: left,
                    b: right,
                    op
                }),
            },
            _ => bail!(TypeckError::InvalidBinaryOp {
                src: self.module.source.clone(),
                span: location.span.into(),
                a: left,
                b: right,
                op
            }),
        }
    }

    /// Infers the type of binary expression.
    ///
    /// This function:
    /// - Infers types of both the left and right operands.
    /// - Checks whether the operator is applicable to the operand types.
    /// - Performs type-level computation (e.g., boolean logic).
    /// - Produces the resulting type, or emits a `TypeckError::InvalidBinaryOp`
    ///   if operands are incompatible with the operator.
    ///
    /// # Parameters
    /// - `location`: Source code address of the binary operator.
    /// - `op`: Binary operator being applied.
    /// - `left`: Left-hand side expression.
    /// - `right`: Right-hand side expression.
    ///
    /// # Returns
    /// - The resulting `Typ` after applying the operator.
    ///
    /// # Errors
    /// - [`InvalidBinaryOp`]: when operand types do not match operator requirements.
    ///
    /// # Notes
    /// This function handles:
    /// - String concatenation (`<>`)
    /// - Arithmetic operators (`+`, `-`, `*`, `/`, `%`, `&`, `|`)
    /// - Logical operators (`&&`, `||`, `^`)
    /// - Comparison operators (`<`, `<=`, `>`, `>=`)
    /// - Equality (`==`, `!=`)
    ///
    fn infer_binary(
        &mut self,
        location: Address,
        op: BinaryOp,
        left: Expression,
        right: Expression,
    ) -> Typ {
        // Inferencing left and right types
        let left = self.infer_expr(left);
        let right = self.infer_expr(right);

        // Matching operator
        match op {
            // Concat
            BinaryOp::Concat => self.infer_binary_concat(location, left, right),
            // Arithmetical
            BinaryOp::Add
            | BinaryOp::Sub
            | BinaryOp::Mul
            | BinaryOp::Div
            | BinaryOp::BitwiseAnd
            | BinaryOp::BitwiseOr
            | BinaryOp::Mod => self.infer_binary_arithmetical(location, left, op, right),
            // Logical
            BinaryOp::Xor | BinaryOp::And | BinaryOp::Or => {
                self.infer_binary_logical(location, left, op, right)
            }
            // Compare
            BinaryOp::Ge | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Lt => {
                self.infer_binary_compare(location, left, op, right)
            }
            // Equality
            BinaryOp::Eq | BinaryOp::NotEq => Typ::Prelude(PreludeType::Bool),
        }
    }

    /// Infers the type of as expression.
    ///
    /// This function:
    /// - Infers value type, infers type annotation.
    /// - Checks both types are primitives.
    /// - Checks cast possibility.
    /// - Produces the resulting type
    ///
    /// # Parameters
    /// - `location`: Source code address of the binary operator.
    /// - `op`: Binary operator being applied.
    /// - `left`: Left-hand side expression.
    /// - `right`: Right-hand side expression.
    ///
    /// # Returns
    /// - The resulting `Typ` after applying the operator.
    ///
    /// # Errors
    /// - [`CouldNotCast`]: if both types are incompatible
    /// - [`InvalidAsOp`]: if one or both operands are not primitives
    ///
    ///  TODO: revamp with new coercion rule.
    ///
    fn infer_as(&mut self, location: Address, value: Expression, typ: TypePath) -> Typ {
        // Inferencing left and right types
        let value = self.infer_expr(value);
        let typ = self.infer_type_annotation(typ);

        // Checking both are primitives
        match (value, typ) {
            (Typ::Prelude(value), Typ::Prelude(typ)) => match (value, typ) {
                (PreludeType::Int, PreludeType::Int) => Typ::Prelude(PreludeType::Int),
                (PreludeType::Int, PreludeType::Float) => Typ::Prelude(PreludeType::Float),
                (PreludeType::Float, PreludeType::Float) => Typ::Prelude(PreludeType::Float),
                (PreludeType::Bool, PreludeType::Bool) => Typ::Prelude(PreludeType::Bool),
                (PreludeType::String, PreludeType::String) => Typ::Prelude(PreludeType::String),
                (a, b) => bail!(TypeckError::CouldNotCast {
                    src: self.module.source.clone(),
                    span: location.span.into(),
                    a: Typ::Prelude(a),
                    b: Typ::Prelude(b)
                }),
            },
            (a, b) => bail!(TypeckError::InvalidAsOp {
                src: self.module.source.clone(),
                span: location.span.into(),
                a,
                b
            }),
        }
    }

    /// Infers the type of unary expression.
    ///
    /// This function:
    /// - Infers the type of the operand.
    /// - Checks whether the operator is applicable to the operand types.
    /// - Returns the resulting type, or emits a `TypeckError::InvalidUnaryOp`
    ///   if the operator cannot be applied.
    ///
    /// # Parameters
    /// - `location`: Source location of the unary operator.
    /// - `op`: Unary operator (`-` or `!`).
    /// - `value`: Operand expression.
    ///
    /// # Returns
    /// - The resulting `Typ` after applying the operator.
    ///
    /// # Errors
    /// - [`InvalidUnaryOp`]: operand type does not match operator expectation.
    ///
    /// # Notes
    /// - `-` is valid only for `Int` and `Float`.
    /// - `!` is valid only for `Bool`.
    ///
    fn infer_unary(&mut self, location: Address, op: UnaryOp, value: Expression) -> Typ {
        // Inferencing value
        let inferred_value = self.infer_expr(value);

        // Checking type is prelude
        let value_typ = match &inferred_value {
            Typ::Prelude(t) => t,
            _ => bail!(TypeckError::InvalidUnaryOp {
                src: self.module.source.clone(),
                span: location.span.into(),
                t: inferred_value,
                op
            }),
        };

        // Matching operator
        match op {
            // Negate `-`
            UnaryOp::Neg => match value_typ {
                PreludeType::Int => Typ::Prelude(PreludeType::Int),
                PreludeType::Float => Typ::Prelude(PreludeType::Float),
                _ => bail!(TypeckError::InvalidUnaryOp {
                    src: self.module.source.clone(),
                    span: location.span.into(),
                    t: inferred_value,
                    op
                }),
            },
            // Bool negate / bang `!`
            UnaryOp::Bang => match value_typ {
                PreludeType::Bool => Typ::Prelude(PreludeType::Bool),
                _ => bail!(TypeckError::InvalidUnaryOp {
                    src: self.module.source.clone(),
                    span: location.span.into(),
                    t: inferred_value,
                    op
                }),
            },
        }
    }

    /// Resolves a variable or module symbol by name.
    ///
    /// # Parameters
    /// - `location`: Location of the variable reference.
    /// - `name`: Identifier being resolved.
    ///
    /// # Returns
    /// - A `Res` representing a fully resolved symbol (value, type, module, etc.).
    ///
    /// # Errors
    /// Emitted indirectly through `resolver.resolve` when a symbol is not found.
    ///
    fn infer_get(&self, location: Address, name: EcoString) -> Res {
        self.resolver.resolve(&location, &name)
    }

    /// Resolves a field access on a module (e.g. `Module.field`).
    ///
    /// This function:
    /// - Locates the target module.
    /// - Locates the requested field inside the module.
    /// - Checks visibility (`Public`, `Private`).
    /// - Produces the correct `Res` variant depending on the field kind:
    ///     - `Type`  → `Res::Custom`
    ///     - `Const` → `Res::Value`
    ///     - `Function` → `Res::Value` containing a function type.
    ///
    /// # Parameters
    /// - `field_module`: Name of the module.
    /// - `field_location`: Source location of the field access.
    /// - `field_name`: Name of the field inside the module.
    ///
    /// # Returns
    /// - Resolved field as `Res`.
    ///
    /// # Errors
    /// - [`ModuleIsNotDefined`]: when the module could not be resolved.
    /// - [`ModuleFieldIsNotDefined`]: when the module field is not defined.
    /// - [`ModuleFieldIsPrivate`]: when the module field is private.
    ///
    fn infer_module_field_access(
        &self,
        field_module: EcoString,
        field_location: Address,
        field_name: EcoString,
    ) -> Res {
        // Getting module
        match self.resolver.imported_modules.get(&field_module) {
            // Getting module
            Some(module) => match module.fields.get(&field_name) {
                // If field exists
                // checking its publicity
                Some(def) => match def {
                    ModuleDef::Type(ty) => {
                        match ty.publicity {
                            // If field is public, we resolved field
                            Publicity::Public => Res::Custom(ty.value.clone()),
                            // Else, raising `module field is private`
                            _ => bail!(TypeckError::ModuleFieldIsPrivate {
                                src: self.module.source.clone(),
                                span: field_location.span.into(),
                                name: field_name
                            }),
                        }
                    }
                    ModuleDef::Const(var) => {
                        match var.publicity {
                            // If constant is public, we resolved field
                            Publicity::Public => Res::Value(var.value.clone()),
                            // Else, raising `module field is private`
                            _ => bail!(TypeckError::ModuleFieldIsPrivate {
                                src: self.module.source.clone(),
                                span: field_location.span.into(),
                                name: field_name
                            }),
                        }
                    }
                    ModuleDef::Function(f) => {
                        match f.publicity {
                            // If constant is public, we resolved field
                            Publicity::Public => Res::Value(Typ::Function(f.value.clone())),
                            // Else, raising `module field is private`
                            _ => bail!(TypeckError::ModuleFieldIsPrivate {
                                src: self.module.source.clone(),
                                span: field_location.span.into(),
                                name: field_name
                            }),
                        }
                    }
                },
                // Else, raising `module field is not defined`
                None => bail!(TypeckError::ModuleFieldIsNotDefined {
                    src: self.module.source.clone(),
                    span: field_location.span.into(),
                    m: field_module,
                    field: field_name
                }),
            },
            // If module is not defined
            None => bail!(TypeckError::ModuleIsNotDefined { m: field_module }),
        }
    }

    /// Resolves a field access on an enum type (variant lookup).
    ///
    /// This function:
    /// - Retrieves the list of variants from the enum definition.
    /// - Searches for a variant with the requested name.
    /// - Returns `Res::Variant` on success.
    ///
    /// # Parameters
    /// - `ty`: Fully instantiated enum type.
    /// - `name`: Name of the enum type (used for error reporting).
    /// - `field_location`: Source location.
    /// - `field_name`: Name of the variant being accessed.
    ///
    /// # Returns
    /// - `Res::Variant(ty, variant)`
    ///
    /// # Errors
    /// - [`FieldIsNotDefined`]: the variant does not exist in the enum.
    ///
    fn infer_enum_field_access(
        &mut self,
        ty: Typ,
        name: EcoString,
        field_location: Address,
        field_name: EcoString,
    ) -> Res {
        // Finding field
        match ty
            .variants(&mut self.solver.hydrator)
            .iter()
            .find(|f| f.name == field_name)
        {
            Some(f) => Res::Variant(ty, f.clone()),
            None => bail!(TypeckError::FieldIsNotDefined {
                src: self.module.source.clone(),
                span: field_location.span.into(),
                t: name,
                field: field_name
            }),
        }
    }

    /// Resolves a field access on a struct type.
    ///
    /// This function:
    /// - Retrieves struct fields via the hydrator.
    /// - Searches for a field with the requested name.
    /// - Returns the type of the field inside `Res::Value`.
    ///
    /// # Parameters
    /// - `ty`: Fully instantiated struct type.
    /// - `name`: Struct name for error reporting.
    /// - `field_location`: Source code address.
    /// - `field_name`: Field name.
    ///
    /// # Returns
    /// - `Res::Value(f.typ)` if field exists.
    ///
    /// # Errors
    /// - [`FieldIsNotDefined`]: the field does not exist in the struct.
    ///
    fn infer_struct_field_access(
        &mut self,
        ty: Typ,
        name: EcoString,
        field_location: Address,
        field_name: EcoString,
    ) -> Res {
        // Finding field
        match ty
            .fields(&mut self.solver.hydrator)
            .iter()
            .find(|f| f.name == field_name)
        {
            Some(f) => Res::Value(f.typ.clone()),
            None => bail!(TypeckError::FieldIsNotDefined {
                src: self.module.source.clone(),
                span: field_location.span.into(),
                t: name,
                field: field_name
            }),
        }
    }

    /// Infers any kind of field access expression.
    ///
    /// Depending on what the container resolves to, this function does this:
    ///
    /// - calls                        `infer_module_field_access`  for module fields
    /// - instantiates enum and calls  `infer_enum_field_access`    for enum variants
    /// - calls                        `infer_struct_field_access`  for struct value fields
    ///
    /// # Parameters
    /// - `field_location`: Location of the field access.
    /// - `container`: Expression on the left-hand side of `.`.
    /// - `field_name`: Requested field.
    ///
    /// # Returns
    /// - `Res` representing the resolved field.
    ///
    /// # Errors
    /// - [`CouldNotResolveFieldsIn`]: container is not a module/struct/enum.
    ///
    fn infer_field_access(
        &mut self,
        field_location: Address,
        container: Expression,
        field_name: EcoString,
    ) -> Res {
        // Inferring container
        let container_inferred = self.infer_resolution(container);
        match &container_inferred {
            // Module field access
            Res::Module(name) => {
                self.infer_module_field_access(name.clone(), field_location, field_name)
            }
            // Enum field access
            Res::Custom(TypeDef::Enum(en)) => {
                let instantiated = Typ::Enum(
                    en.clone(),
                    self.solver
                        .hydrator
                        .hyd()
                        .mk_generics(&en.borrow().generics, HashMap::new()),
                );
                self.infer_enum_field_access(
                    instantiated,
                    en.borrow().name.clone(),
                    field_location,
                    field_name,
                )
            }
            // Type field access
            Res::Value(it @ Typ::Struct(ty, _)) => self.infer_struct_field_access(
                it.clone(),
                ty.borrow().name.clone(),
                field_location,
                field_name,
            ),
            // Else
            _ => bail!(TypeckError::CouldNotResolveFieldsIn {
                src: self.module.source.clone(),
                span: field_location.span.into(),
                res: container_inferred
            }),
        }
    }

    /// Ensures arity of parameters and arguments.
    ///
    /// # Parameters
    /// - `location`: Location of the field access.
    /// - `expected`: Expected amount of parameters.
    /// - `got`: Amount of passed parameters.
    fn ensure_arity(&self, location: Address, expected: usize, got: usize) {
        if expected != got {
            bail!(TypeckError::ArityMissmatch {
                related: vec![TypeckRelated::Here {
                    src: location.source.clone(),
                    span: location.span.into()
                }],
                expected,
                got
            })
        }
    }

    /// Infers the type of function or constructor call.
    ///
    /// This routine performs three major tasks:
    /// 1. Resolves the callee (`what`) via [`infer_resolution`] to determine whether it is:
    ///    - a function,
    ///    - a struct constructor,
    ///    - an enum variant,
    ///    - or an invalid expression,
    /// 2. Infers the types of all argument expressions,
    /// 3. Produces unification constraints between the expected and provided argument types.
    ///
    /// ### Struct constructor call
    /// If the callee resolves to a custom struct type (`Res::Custom(TypeDef::Struct)`),
    /// the hydrator instantiates its generic parameters with fresh variables.
    /// Then each struct field type is unified with the corresponding argument.
    ///
    /// ### Function call
    /// If the callee resolves to a function (`Typ::Function`), the function signature
    /// is instantiated via [`Hydrator::mk_function`] and each parameter is unified with
    /// the corresponding argument expression.
    ///
    /// ### Enum variant construction
    /// If the callee is an enum variant (`Res::Variant`), each variant field is unified
    /// with its corresponding argument expression. We don't instantiate the enum,
    /// because it was already instantiated during enum variant / enum field lookup.
    ///
    /// ### Errors
    /// - [`TypeckError::CouldNotCall`]: the callee is not callable (e.g. an integer).
    /// - ['TypeckError::ArityMismatch`]: or type mismatches are detected via solver unification.
    ///
    /// Returns a resolved `Res::Value` with the instantiated type of the expression.
    ///
    pub(crate) fn infer_call(
        &mut self,
        location: Address,
        what: Expression,
        args: Vec<Expression>,
    ) -> Res {
        let function = self.infer_resolution(what);
        let args = args
            .iter()
            .map(|a| (a.location(), self.infer_expr(a.clone())))
            .collect::<Vec<(Address, Typ)>>();

        match function.clone() {
            // Custom type
            Res::Custom(TypeDef::Struct(ty)) => {
                self.ensure_arity(location, ty.borrow().fields.len(), args.len());

                let instantiated = Typ::Struct(
                    ty.clone(),
                    self.solver
                        .hydrator
                        .hyd()
                        .mk_generics(&ty.borrow().generics, HashMap::new()),
                );

                instantiated
                    .fields(&mut self.solver.hydrator)
                    .into_iter()
                    .zip(args)
                    .for_each(|(p, a)| {
                        self.solver
                            .solve(Equation::Unify(EqUnit(p.location, p.typ), EqUnit(a.0, a.1)));
                    });

                Res::Value(instantiated)
            }
            // Value
            Res::Value(Typ::Function(f)) => {
                self.ensure_arity(location, f.params.len(), args.len());
                let f = self.solver.hydrator.hyd().mk_function(f);
                f.params.iter().cloned().zip(args).for_each(|(p, a)| {
                    self.solver
                        .solve(Equation::Unify(EqUnit(p.location, p.typ), EqUnit(a.0, a.1)));
                });
                Res::Value(f.ret.clone())
            }
            // Variant
            Res::Variant(en, variant) => {
                variant.fields.iter().cloned().zip(args).for_each(|(p, a)| {
                    self.solver
                        .solve(Equation::Unify(EqUnit(p.location, p.typ), EqUnit(a.0, a.1)));
                });

                Res::Value(en)
            }
            _ => bail!(TypeckError::CouldNotCall {
                src: self.module.source.clone(),
                span: location.span.into(),
                res: function
            }),
        }
    }

    /// Performs name/field resolution on an expression that appears in a "call position".
    ///
    /// This function is responsible only for *resolving what the expression refers to*.
    /// It does **not** infer full expression types (that's [`infer_expr`]).
    ///
    /// Supported resolution forms:
    /// - `PrefixVar`: simple variable access,
    /// - `SuffixVar`: field access (`a.b`),
    /// - nested calls (`f(x)(y)`), which recursively call [`infer_call`].
    ///
    /// Any other expression that cannot denote a callable value or a namespace entry
    /// triggers [`TypeckError::UnexpectedExprInResolution`].
    ///
    /// This function is typically used at the entry point of call inference
    /// and pattern matching, where the compiler needs to know *what* is being referenced.
    ///
    pub(crate) fn infer_resolution(&mut self, expr: Expression) -> Res {
        match expr {
            Expression::PrefixVar { location, name } => self.infer_get(location, name),
            Expression::SuffixVar {
                location,
                container,
                name,
            } => self.infer_field_access(location, *container, name),
            Expression::Call {
                location,
                what,
                args,
            } => self.infer_call(location.clone(), *what, args),
            expr => bail!(TypeckError::UnexpectedExprInResolution {
                expr: format!("{expr:?}").into()
            }),
        }
    }

    /// Infers the type of anonymous function literal.
    ///
    /// This creates a temporary local scope, binds parameters with their declared
    /// annotated types, and infers the type of the function body.
    ///
    /// ### Return type
    /// - If an explicit return type is provided, it is used.
    /// - Otherwise the return type defaults to `Unit`, but is unified with the inferred body.
    ///
    /// ### Parameters
    /// Parameter types must always be annotated; the inference engine does not attempt
    /// to infer parameter types from usage (similar to Rust).
    ///
    /// ### Scoping
    /// A new rib (scope) is pushed for the function parameters. After the body is
    /// inferred and unified, the rib is popped.
    ///
    /// Returns a fully constructed `Typ::Function` containing:
    /// - inferred parameter list,
    /// - inferred return type,
    /// - captured generics (**Always** empty for anonymous functions).
    ///
    fn infer_anonymous_fn(
        &mut self,
        location: Address,
        params: Vec<ast::Parameter>,
        body: Either<Block, Box<Expression>>,
        ret_type: Option<TypePath>,
    ) -> Typ {
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

        // creating function
        let function = Function {
            location: location.clone(),
            name: EcoString::from("$anonymous"),
            generics: Vec::new(),
            params: params.clone().into_values().collect(),
            ret: ret.clone(),
        };

        // pushing new scope
        self.resolver.push_rib();

        // defining params in new scope
        params
            .into_iter()
            .for_each(|p| self.resolver.define_local(&location, &p.0, p.1.typ, false));

        // inferring body
        let (block_location, inferred_block) = match body {
            Either::Left(block) => (block.location.clone(), self.infer_block(block)),
            Either::Right(expr) => (expr.location(), self.infer_expr(*expr)),
        };
        self.solver.solve(Equation::Unify(
            EqUnit(location, ret),
            EqUnit(block_location, inferred_block),
        ));
        self.resolver.pop_rib();

        // result
        Typ::Function(Rc::new(function))
    }

    /// Performs semantic/type analysis of a single match arm pattern.
    ///
    /// Validates the correctness of a pattern
    /// against the expected type of the matched value (`inferred_what`).
    ///
    /// ### Responsibilities:
    /// - Verifies enum variant constructors used in patterns,
    /// - Verifies the correctness of fields in an `Unwrap` pattern,
    /// - Ensures literals (`Int`, `Float`, etc.) match the expected type,
    /// - Handles wildcards (`_`) and variable binding patterns,
    /// - Recursively validates `pat1 | pat2`
    ///
    /// ### Errors:
    /// - [`TypeckError::TypesMissmatch`] — literal or variant does not match the scrutinee type.
    /// - [`TypeckError::WrongUnwrapPattern`] — using `.field` pattern on non-variant.
    /// - [`TypeckError::EnumVariantFieldIsNotDefined`] — non-existent field in variant.
    /// - [`TypeckError::WrongVariantPattern`] — non-variant used where variant pattern expected.
    ///
    /// This function may introduce new local bindings (for `BindTo`) into the current rib.
    ///
    fn analyze_pattern(&mut self, inferred_what: Typ, case: &Case, pat: &Pattern) {
        // matching pattern
        match pat.clone() {
            Pattern::Unwrap { en, fields } => {
                // inferring resolution, and checking
                // that is an enum variant
                let res = self.infer_resolution(en);
                match &res {
                    Res::Variant(en, variant) => {
                        // If types aren't equal
                        if inferred_what != *en {
                            bail!(TypeckError::TypesMissmatch {
                                src: self.module.source.clone(),
                                span: case.address.span.clone().into(),
                                expected: en.clone(),
                                got: inferred_what.clone()
                            });
                        }
                        // If types equal, checking fields existence
                        else {
                            fields.into_iter().for_each(|field| {
                                // Defining fields and checking existence
                                match variant.fields.iter().find(|f| f.name == field.1) {
                                    // Note: Don't worry about field type instantiation,
                                    // it was already instantiated by instantiating the enum
                                    // itself and getting fresh enum variant
                                    // during variant resolution.
                                    Some(it) => self.resolver.define_local(
                                        &case.address,
                                        &it.name,
                                        it.typ.clone(),
                                        false,
                                    ),
                                    None => bail!(TypeckError::EnumVariantFieldIsNotDefined {
                                        src: self.module.source.clone(),
                                        span: field.0.span.into(),
                                        res: res.clone(),
                                        field: field.1
                                    }),
                                }
                            });
                        }
                    }
                    _ => bail!(TypeckError::WrongUnwrapPattern {
                        src: self.module.source.clone(),
                        span: case.address.span.clone().into(),
                        got: res
                    }),
                }
            }
            Pattern::Int(_) => {
                let typ = Typ::Prelude(PreludeType::Int);
                if inferred_what != typ {
                    bail!(TypeckError::TypesMissmatch {
                        src: self.module.source.clone(),
                        span: case.address.span.clone().into(),
                        expected: inferred_what,
                        got: typ
                    })
                }
            }
            Pattern::Float(_) => {
                let typ = Typ::Prelude(PreludeType::Float);
                if inferred_what != typ {
                    bail!(TypeckError::TypesMissmatch {
                        src: self.module.source.clone(),
                        span: case.address.span.clone().into(),
                        expected: inferred_what,
                        got: typ
                    })
                }
            }
            Pattern::String(_) => {
                let typ = Typ::Prelude(PreludeType::String);
                if inferred_what != typ {
                    bail!(TypeckError::TypesMissmatch {
                        src: self.module.source.clone(),
                        span: case.address.span.clone().into(),
                        expected: inferred_what,
                        got: typ
                    })
                }
            }
            Pattern::Bool(_) => {
                let typ = Typ::Prelude(PreludeType::Bool);
                if inferred_what != typ {
                    bail!(TypeckError::TypesMissmatch {
                        src: self.module.source.clone(),
                        span: case.address.span.clone().into(),
                        expected: inferred_what,
                        got: typ
                    })
                }
            }
            Pattern::Wildcard => skip!(),
            Pattern::Variant(var) => {
                // inferring resolution, and checking
                // that is an enum variant
                let res = self.infer_resolution(var);
                match &res {
                    Res::Variant(en, _) => {
                        // If types aren't equal
                        if inferred_what != *en {
                            bail!(TypeckError::TypesMissmatch {
                                src: self.module.source.clone(),
                                span: case.address.span.clone().into(),
                                expected: en.clone(),
                                got: inferred_what.clone()
                            });
                        }
                    }
                    _ => bail!(TypeckError::WrongVariantPattern {
                        src: self.module.source.clone(),
                        span: case.address.span.clone().into(),
                        got: res
                    }),
                }
            }
            Pattern::BindTo(name) => {
                self.resolver
                    .define_local(&case.address, &name, inferred_what.clone(), false);
            }
            Pattern::Or(pat1, pat2) => {
                self.analyze_pattern(inferred_what.clone(), case, &pat1);
                self.analyze_pattern(inferred_what, case, &pat2);
            }
        }
    }

    /// Infers the result type of `match` expression.
    ///
    /// Steps performed:
    /// 1. Infer the matchable type (`inferred_what`).
    /// 2. For each case:
    ///    - push a new rib,
    ///    - analyze its pattern via [`analyze_pattern`],
    ///    - infer the type of its body,
    ///    - collect all case body types for unification,
    ///    - pop the rib.
    /// 3. Unify all case body types yielding the final type of the `match`.
    /// 4. Perform exhaustiveness checking using [`ExMatchCx::check`].
    ///
    /// ### Exhaustiveness
    /// If the match is not exhaustive:
    /// - Emit a warning (`TypeckWarning::NonExhaustive`),
    /// - The whole match expression is typed as `Unit`.
    ///
    /// Otherwise, return the unified type of all branches.
    ///
    pub(crate) fn infer_pattern_matching(
        &mut self,
        location: Address,
        what: Expression,
        cases: Vec<Case>,
    ) -> Typ {
        // inferring matchable
        let inferred_what = self.infer_expr(what);
        // to unify
        let mut to_unify = Vec::new();
        // type checking cases
        for case in cases.clone() {
            // pattern scope start
            self.resolver.push_rib();
            // analyzing pattern
            self.analyze_pattern(inferred_what.clone(), &case, &case.pattern);
            // analyzing body
            let (case_location, inferred_case) = match case.body {
                Either::Left(block) => (block.location.clone(), self.infer_block(block)),
                Either::Right(expr) => (expr.location(), self.infer_expr(expr)),
            };
            to_unify.push(EqUnit(case_location, inferred_case));
            // pattern scope end
            self.resolver.pop_rib();
        }
        // solved type
        let typ = self.solver.solve(Equation::UnifyMany(to_unify));
        let checked = ExMatchCx::check(self, inferred_what, cases);
        // checking all cases covered
        if checked {
            typ
        } else {
            warn!(
                self.package,
                TypeckWarning::NonExhaustive {
                    src: location.source,
                    span: location.span.into()
                }
            );
            Typ::Unit
        }
    }

    /// Infers the type of `if`/`elif`/`else` chain.
    ///
    /// ### Logical expression
    /// Ensures that each `if` and `elif` condition has type `Bool`.
    /// Otherwise, emits [`TypeckError::ExpectedLogicalInIf`].
    ///
    /// ### Branch types
    /// All reachable branches are collected into a list and unified together.
    /// If the final `else` branch is missing, the whole `if` expression evaluates to `Unit`.
    ///
    /// ### Scoping
    /// Each `if` branch introduces a new rib; scoping is handled consistently like in blocks.
    ///
    /// Returns:
    /// - The unified type of all branches if an `else` exists,
    /// - Otherwise `Unit`.
    ///
    fn infer_if(
        &mut self,
        location: Address,
        logical: Expression,
        body: Either<Block, Box<Expression>>,
        else_branches: Vec<ElseBranch>,
    ) -> Typ {
        // pushing rib
        self.resolver.push_rib();
        // inferring logical
        let inferred_logical = self.infer_expr(logical);
        match inferred_logical {
            Typ::Prelude(PreludeType::Bool) => {}
            _ => {
                bail!(TypeckError::ExpectedLogicalInIf {
                    src: self.module.source.clone(),
                    span: location.span.into()
                })
            }
        }
        // inferring block
        let (if_location, inferred_if) = match body {
            Either::Left(block) => (block.location.clone(), self.infer_block(block)),
            Either::Right(expr) => (expr.location(), self.infer_expr(*expr)),
        };
        let mut to_unify = vec![EqUnit(if_location, inferred_if)];
        // popping rib
        self.resolver.pop_rib();
        // else reached
        let mut else_reached = false;
        // analyzing else branches
        for branch in else_branches {
            match branch {
                ElseBranch::Elif { logical, body, .. } => {
                    // inferring logical
                    let logical_location = logical.location();
                    let inferred_logical = self.infer_expr(logical);
                    match inferred_logical {
                        Typ::Prelude(PreludeType::Bool) => {}
                        _ => {
                            bail!(TypeckError::ExpectedLogicalInIf {
                                src: self.module.source.clone(),
                                span: logical_location.span.into()
                            })
                        }
                    }
                    // inferring block
                    let (branch_location, inferred_branch) = match body {
                        Either::Left(block) => (block.location.clone(), self.infer_block(block)),
                        Either::Right(expr) => (expr.location(), self.infer_expr(expr)),
                    };
                    to_unify.push(EqUnit(branch_location, inferred_branch));
                }
                ElseBranch::Else { body, .. } => {
                    // inferring block
                    let (branch_location, inferred_branch) = match body {
                        Either::Left(block) => (block.location.clone(), self.infer_block(block)),
                        Either::Right(expr) => (expr.location(), self.infer_expr(expr)),
                    };
                    to_unify.push(EqUnit(branch_location, inferred_branch));
                    else_reached = true;
                }
            }
        }
        // checking else reached
        if else_reached {
            self.solver.solve(Equation::UnifyMany(to_unify))
        } else {
            Typ::Unit
        }
    }

    /// The central entry point for expression type inference.
    ///
    /// Dispatches to specialized inference routines depending on expression kind:
    /// - literals → primitive `PreludeType`,
    /// - variable and field access,
    /// - calls (`infer_call`),
    /// - anonymous functions (`infer_anonymous_fn`),
    /// - binary and unary ops,
    /// - match/if constructs.
    ///
    /// After the initial inference, the result is passed through the hydrator
    /// (`Hydrator::apply`) to resolve any pending substitutions of unbounds.
    ///
    /// This guarantees that the final type is always normalized.
    ///
    pub(crate) fn infer_expr(&mut self, expr: Expression) -> Typ {
        // Inferencing expression
        let result = match expr {
            Expression::Float { .. } => Typ::Prelude(PreludeType::Float),
            Expression::Int { .. } => Typ::Prelude(PreludeType::Int),
            Expression::String { .. } => Typ::Prelude(PreludeType::String),
            Expression::Bool { .. } => Typ::Prelude(PreludeType::Bool),
            Expression::Todo { location, .. } => {
                warn!(
                    self.package,
                    TypeckWarning::FoundTodo {
                        src: self.module.source.clone(),
                        span: location.span.into()
                    }
                );
                Typ::Unbound(self.solver.hydrator.fresh())
            }
            Expression::Panic { .. } => Typ::Unbound(self.solver.hydrator.fresh()),
            Expression::Bin {
                location,
                left,
                right,
                op,
            } => self.infer_binary(location, op, *left, *right),
            Expression::As {
                location,
                value,
                typ,
            } => self.infer_as(location, *value, typ),
            Expression::Unary {
                location,
                value,
                op,
            } => self.infer_unary(location, op, *value),
            Expression::PrefixVar { location, name } => {
                self.infer_get(location.clone(), name).unwrap_typ(&location)
            }
            Expression::SuffixVar {
                location,
                container,
                name,
            } => self
                .infer_field_access(location.clone(), *container, name)
                .unwrap_typ(&location),
            Expression::Call {
                location,
                what,
                args,
            } => self
                .infer_call(location.clone(), *what, args)
                .unwrap_typ(&location),
            Expression::Function {
                location,
                params,
                body,
                typ,
            } => self.infer_anonymous_fn(location, params, body, typ),
            Expression::Match {
                location,
                value,
                cases,
                ..
            } => self.infer_pattern_matching(location, *value, cases),
            Expression::If {
                location,
                logical,
                body,
                else_branches,
            } => self.infer_if(location, *logical, body, else_branches),
        };
        // Applying substs
        self.solver.hydrator.apply(result)
    }
}
