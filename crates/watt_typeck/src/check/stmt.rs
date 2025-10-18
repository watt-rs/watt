/// Imports
use crate::{
    cx::module::ModuleCx,
    errors::TypeckError,
    resolve::{resolve::Def, rib::RibKind},
    typ::{Function, PreludeType, Typ},
    unify::Equation,
};
use ecow::EcoString;
use std::collections::HashMap;
use watt_ast::ast::*;
use watt_ast::ast::{Block, Expression, Parameter, TypePath};
use watt_common::{address::Address, bail, rc_ptr::RcPtr};

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

    /// Analyzes define
    pub(crate) fn analyze_define(
        &mut self,
        location: Address,
        name: EcoString,
        value: Expression,
        typ: Option<TypePath>,
    ) {
        let inferred_value = self.infer_expr(value);
        match typ {
            Some(annotated_path) => {
                let annotated_location = annotated_path.get_location();
                let annotated = self.infer_type_annotation(annotated_path);
                self.solver.solve(Equation::Unify(
                    (location.clone(), inferred_value.clone()),
                    (annotated_location, annotated),
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

    /// Analyzes funciton statement
    fn analyze_function(
        &mut self,
        location: Address,
        name: EcoString,
        params: Vec<Parameter>,
        body: Block,
        ret_type: Option<TypePath>,
    ) {
        // inferring return type
        let ret = ret_type.map_or(Typ::Unit, |t| self.infer_type_annotation(t));

        // inferring params
        let params = params
            .into_iter()
            .map(|p| (p.name, self.infer_type_annotation(p.typ.clone())))
            .collect::<HashMap<EcoString, Typ>>();

        // creating and defining function
        let function = Function {
            source: self.module.source.clone(),
            location: location.clone(),
            name: name.clone(),
            params: params.clone().into_values().collect::<Vec<Typ>>(),
            ret: ret.clone(),
        };
        self.resolver.define(
            &self.module.source.clone(),
            &location,
            &name,
            Def::Local(Typ::Function(RcPtr::new(function))),
        );

        // pushing new scope
        self.resolver.push_rib(RibKind::Function);

        // defining params in new scope
        params.iter().for_each(|p| {
            self.resolver
                .define(&self.module.source, &location, p.0, Def::Local(p.1.clone()))
        });

        // inferring body
        let block_location = body.location.clone();
        let inferred_block = self.infer_block(body);
        self.solver.solve(Equation::Unify(
            (location, ret),
            (block_location, inferred_block),
        ));
        self.resolver.pop_rib();
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
            match stmt {
                Statement::VarDef {
                    location,
                    name,
                    value,
                    typ,
                } => self.analyze_define(location, name, value, typ),
                Statement::VarAssign {
                    location,
                    what,
                    value,
                } => self.analyze_assignment(location, what, value),
                Statement::Expr(expression) => {
                    self.infer_expr(expression);
                }
                Statement::While {
                    location,
                    logical,
                    body,
                } => self.analyze_while(location, logical, body),
                Statement::Break { .. } => {
                    if !self.resolver.contains_rib(RibKind::Loop) {
                        unimplemented!()
                    }
                }
                Statement::Continue { .. } => {
                    if !self.resolver.contains_rib(RibKind::Loop) {
                        unimplemented!()
                    }
                }
            };
        }
        // Inferring last
        match last {
            Statement::Expr(expression) => self.infer_expr(expression),
            _ => Typ::Unit,
        }
    }
}
