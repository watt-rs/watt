/// Imports
use crate::{
    cx::module::ModuleCx,
    errors::TypeckError,
    inference::equation::Equation,
    typ::typ::{PreludeType, Typ},
};
use ecow::EcoString;
use watt_ast::ast::*;
use watt_ast::ast::{Block, Expression, TypePath};
use watt_common::{address::Address, bail};

/// Statements iferring
impl<'pkg, 'cx> ModuleCx<'pkg, 'cx> {
    /// Analyzes loop
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
            Typ::Prelude(PreludeType::Bool) => {}
            _ => bail!(TypeckError::ExpectedLogicalInWhile {
                src: self.module.source.clone(),
                span: location.span.into()
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

    /// Analyzes range
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
                        expected: typ,
                        got: inferred_from
                    })
                }
                if inferred_to != typ {
                    bail!(TypeckError::TypesMissmatch {
                        src: location.source,
                        span: location.span.into(),
                        expected: typ,
                        got: inferred_from
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
                        expected: typ,
                        got: inferred_from
                    })
                }
                if inferred_to != typ {
                    bail!(TypeckError::TypesMissmatch {
                        src: location.source,
                        span: location.span.into(),
                        expected: typ,
                        got: inferred_from
                    })
                }
            }
        }
    }

    /// Analyzes for
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

    /// Analyzes `let` define
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
                self.solver.solve(Equation::Unify(
                    (annotated_location, annotated.clone()),
                    (value_location.clone(), inferred_value.clone()),
                ));
                self.resolver
                    .define_local(&location, &name, annotated, false)
            }
            None => self
                .resolver
                .define_local(&location, &name, inferred_value, false),
        }
    }

    /// Analyzes assignment
    fn analyze_assignment(&mut self, location: Address, what: Expression, value: Expression) {
        let inferred_what = self.infer_expr(what);
        let value_location = value.location();
        let inferred_value = self.infer_expr(value);
        self.solver.solve(Equation::Unify(
            (location, inferred_what),
            (value_location, inferred_value),
        ));
    }

    /// Infers stmt
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
                self.analyze_for(location, name, range, body);
                Typ::Unit
            }
            Statement::Semi(expr) => {
                self.infer_expr(expr);
                Typ::Unit
            }
        }
    }

    /// Infers block
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
