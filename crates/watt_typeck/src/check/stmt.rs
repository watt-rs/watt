/// Imports
use crate::{
    cx::module::ModuleCx,
    errors::TypeckError,
    resolve::{resolve::Def, rib::RibKind},
    typ::{PreludeType, Typ},
    unify::Equation,
};
use ecow::EcoString;
use watt_ast::ast::*;
use watt_ast::ast::{Block, Expression, TypePath};
use watt_common::{address::Address, bail};

/// Statements iferring
impl<'pkg, 'cx> ModuleCx<'pkg, 'cx> {
    /// Analyzes while
    fn analyze_while(&mut self, location: Address, logical: Expression, body: Block) {
        // pushing rib
        self.resolver.push_rib(RibKind::Loop);
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
        let _ = self.infer_block(body);
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
                let annotated_location = annotated_path.get_location();
                let annotated = self.infer_type_annotation(annotated_path);
                self.solver.solve(Equation::Unify(
                    (annotated_location, annotated),
                    (value_location.clone(), inferred_value.clone()),
                ));
                self.resolver.define(
                    &self.module.source,
                    &location,
                    &name,
                    Def::Local(inferred_value),
                )
            }
            None => self.resolver.define(
                &self.module.source,
                &location,
                &name,
                Def::Local(inferred_value),
            ),
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
                self.analyze_while(location, logical, body);
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
