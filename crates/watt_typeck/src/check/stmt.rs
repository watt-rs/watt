/// Imports
use crate::{
    cx::module::ModuleCx,
    errors::TypeckError,
    inference::coercion::{self, Coercion},
    typ::{
        res::Res,
        typ::{PreludeType, Typ},
    },
};
use ecow::EcoString;
use watt_ast::ast::*;
use watt_ast::ast::{Block, Expression, TypePath};
use watt_common::{address::Address, bail, skip};

/// Statements inferencing
impl<'pkg, 'cx> ModuleCx<'pkg, 'cx> {
    /// Performs semantic and type analysis for a `loop` / `while` construct.
    ///
    /// ## Steps:
    /// - Push a new local-scope rib for variables declared inside the loop.
    /// - Infer the type of the loop condition (`logical`) and ensure it is `Bool`.
    /// - Infer the type of the loop body (either a block or single expression).
    /// - Pop the previously pushed rib.
    ///
    /// ## Errors
    /// - [`TypeckError::TypesMissmatch`] if the condition does not have type `Bool`.
    ///
    fn analyze_loop(
        &mut self,
        location: Address,
        logical: Expression,
        body: Either<Block, Expression>,
    ) {
        // pushing rib
        self.resolver.push_rib();
        // inferring logical
        let inferred_logical = self.infer_expr(logical);
        match inferred_logical {
            Typ::Prelude(PreludeType::Bool) => skip!(),
            other => bail!(TypeckError::TypesMissmatch {
                src: self.module.source.clone(),
                span: location.span.into(),
                expected: Typ::Prelude(PreludeType::Bool).pretty(&mut self.icx),
                got: other.pretty(&mut self.icx),
            }),
        }
        // inferring block
        let _ = match body {
            Either::Left(block) => self.infer_block(block),
            Either::Right(expr) => self.infer_expr(expr),
        };
        // popping rib
        self.resolver.pop_rib();
    }

    /// Performs type analysis for a `range` expression used in `for` loops.
    ///
    /// ## Steps:
    /// - Infer the types of both endpoints (`from`, `to`).
    /// - Ensure both have type `Int`, regardless of the range variant
    ///   (`ExcludeLast` / `IncludeLast`).
    ///
    /// ## Errors:
    /// - [`TypeckError::TypesMissmatch`] if any endpoint is not an `Int`.
    ///
    fn analyze_range(&mut self, range: Range) {
        match range {
            Range::ExcludeLast { location, from, to } => {
                // Inferring from and to expression
                let inferred_from = self.infer_expr(from);
                let inferred_to = self.infer_expr(to);
                // Checking both are ints
                let typ = Typ::Prelude(PreludeType::Int);
                if inferred_from != typ {
                    bail!(TypeckError::TypesMissmatch {
                        src: location.source,
                        span: location.span.into(),
                        expected: typ.pretty(&mut self.icx),
                        got: inferred_from.pretty(&mut self.icx)
                    })
                }
                if inferred_to != typ {
                    bail!(TypeckError::TypesMissmatch {
                        src: location.source,
                        span: location.span.into(),
                        expected: typ.pretty(&mut self.icx),
                        got: inferred_from.pretty(&mut self.icx)
                    })
                }
            }
            Range::IncludeLast { location, from, to } => {
                // Inferring from and to expression
                let inferred_from = self.infer_expr(from);
                let inferred_to = self.infer_expr(to);
                // Checking both are ints
                let typ = Typ::Prelude(PreludeType::Int);
                if inferred_from != typ {
                    bail!(TypeckError::TypesMissmatch {
                        src: location.source,
                        span: location.span.into(),
                        expected: typ.pretty(&mut self.icx),
                        got: inferred_from.pretty(&mut self.icx)
                    })
                }
                if inferred_to != typ {
                    bail!(TypeckError::TypesMissmatch {
                        src: location.source,
                        span: location.span.into(),
                        expected: typ.pretty(&mut self.icx),
                        got: inferred_from.pretty(&mut self.icx)
                    })
                }
            }
        }
    }

    /// Performs semantic and type analysis for a `for` loop.
    ///
    /// ## Steps:
    /// - Push a new rib for loop-local bindings.
    /// - Declare the loop variable (`name`) as an `Int` (ranges iterate over integers).
    /// - Analyze the provided `range` using [`analyze_range`].
    /// - Infer the type of the loop body.
    /// - Pop the rib after finishing the loop body analysis.
    ///
    /// # Errors:
    /// Emitted indirectly.
    ///
    fn analyze_for(
        &mut self,
        location: Address,
        name: EcoString,
        range: Range,
        body: Either<Block, Expression>,
    ) {
        // pushing rib
        self.resolver.push_rib();
        // defining variable for iterations
        self.resolver
            .define_local(&location, &name, Typ::Prelude(PreludeType::Int), false);
        // analyzing range
        self.analyze_range(range);
        // inferring block
        let _ = match body {
            Either::Left(block) => self.infer_block(block),
            Either::Right(expr) => self.infer_expr(expr),
        };
        // popping rib
        self.resolver.pop_rib();
    }

    /// Analyzes a `let` variable definition.
    ///
    /// ## Steps:
    /// - Infer the type of the initialization expression (`value`).
    /// - If the declaration includes a type annotation:
    ///     - Infer the annotation type.
    ///     - Emit a unification equation requiring the annotated and inferred
    ///       types to be equal.
    ///     - Define the variable with the annotated type.
    /// - If no annotation was provided:
    ///     - Define the variable using the inferred type.
    ///
    pub(crate) fn analyze_let_define(
        &mut self,
        location: Address,
        name: EcoString,
        value: Expression,
        typ: Option<TypePath>,
    ) {
        let value_location = value.location();
        let inferred_value = self.infer_expr(value);
        match typ {
            Some(annotated_path) => {
                let annotated_location = annotated_path.location();
                let annotated = self.infer_type_annotation(annotated_path);
                let inferred_value = self.icx.mk_fresh(inferred_value);
                coercion::coerce(
                    &mut self.icx,
                    Coercion::Eq(
                        (annotated_location, annotated.clone()),
                        (value_location.clone(), inferred_value.clone()),
                    ),
                );
                self.resolver
                    .define_local(&location, &name, annotated, false)
            }
            None => self
                .resolver
                .define_local(&location, &name, inferred_value, false),
        }
    }

    /// Analyzes an assignment (`x = value`).
    ///
    /// ## Steps:
    /// - Resolve the left-hand side (`what`) and check that it is not a constant.
    /// - Infer the type of the assigned value.
    /// - Emit an equation unifying the variable's type and the value's type.
    ///
    /// ## Errors:
    /// - [`TypeckError::CouldNotAssignConstant`] if the left-hand side refers to a constant.
    ///
    fn analyze_assignment(&mut self, location: Address, what: Expression, value: Expression) {
        let inferred_what = self.infer_resolution(what);
        if let Res::Const(_) = inferred_what {
            bail!(TypeckError::CouldNotAssignConstant {
                src: location.source.clone(),
                span: location.span.into(),
            })
        }
        let value_location = value.location();
        let inferred_value = self.infer_expr(value);
        coercion::coerce(
            &mut self.icx,
            Coercion::Eq(
                (location.clone(), inferred_what.unwrap_typ(&location)),
                (value_location, inferred_value),
            ),
        );
    }

    /// Infers the type of statement.
    ///
    /// ## Behavior by statement kind:
    /// - `Expr` — evaluates to the expression’s type.
    /// - `VarDef` — delegates to [`analyze_let_define`] and returns `Unit`.
    /// - `VarAssign` — delegates to [`analyze_assignment`] and returns `Unit`.
    /// - `Loop` — delegates to [`analyze_loop`] and returns `Unit`.
    /// - `For` — delegates to [`analyze_for`] and returns `Unit`.
    /// - `Semi(expr)` — infers the expression, discards its value, returns `Unit`.
    ///
    fn infer_stmt(&mut self, stmt: Statement) -> Typ {
        match stmt {
            Statement::Expr(expression) => self.infer_expr(expression),
            Statement::VarDef {
                location,
                name,
                value,
                typ,
            } => {
                self.analyze_let_define(location, name, value, typ);
                Typ::Unit
            }
            Statement::VarAssign {
                location,
                what,
                value,
            } => {
                self.analyze_assignment(location, what, value);
                Typ::Unit
            }
            Statement::Loop {
                location,
                logical,
                body,
            } => {
                self.analyze_loop(location, logical, body);
                Typ::Unit
            }
            Statement::For {
                location,
                name,
                range,
                body,
            } => {
                self.analyze_for(location, name, *range, body);
                Typ::Unit
            }
            Statement::Semi(expr) => {
                self.infer_expr(expr);
                Typ::Unit
            }
        }
    }

    /// Infers the type of block.
    ///
    /// ## Rules:
    /// - All statements except the last are treated as having type `Unit`.
    /// - The type of the block is the type of the last statement.
    /// - Empty blocks evaluate to `Unit`.
    ///
    /// ## Implementation:
    /// - Pop the last statement.
    /// - Infer all previous statements via [`infer_stmt`].
    /// - Infer and return the type of the last statement.
    ///
    pub(crate) fn infer_block(&mut self, mut block: Block) -> Typ {
        // Last stmt
        let last = match block.body.pop() {
            Some(last) => last,
            None => return Typ::Unit, // Block is empty
        };
        // Analyzing each statement
        for stmt in block.body {
            self.infer_stmt(stmt);
        }
        // Inferring last
        self.infer_stmt(last)
    }
}
