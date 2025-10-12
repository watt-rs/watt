/// Imports
use crate::{
    cx::module::ModuleCx,
    errors::TypeckError,
    resolve::{resolve::Def, rib::RibKind},
    typ::{Function, PreludeType, Typ},
};
use ecow::EcoString;
use oil_ast::ast::TypePath;
use oil_common::{address::Address, bail, rc_ptr::RcPtr};
use oil_ir::ir::{IrBlock, IrElseBranch, IrExpression, IrParameter, IrStatement};
use std::collections::HashMap;

/// Statements iferring
impl<'pkg, 'cx> ModuleCx<'pkg, 'cx> {
    /// Infers if
    fn infer_if(
        &mut self,
        location: Address,
        logical: IrExpression,
        body: IrBlock,
        else_branches: Vec<IrElseBranch>,
    ) -> Option<Typ> {
        // pushing rib
        self.resolver.push_rib(RibKind::Conditional);
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
        let if_location = body.get_location();
        let mut expected = self.infer_block(body)?;
        // popping rib
        self.resolver.pop_rib();
        // analyzing else branches
        for branch in else_branches {
            match branch {
                IrElseBranch::Elif { logical, body, .. } => {
                    // inferring logical
                    let logical_location = logical.get_location();
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
                    let branch_location = body.get_location();
                    let inferred = self.infer_block(body)?;
                    expected = self.unify(&if_location, &expected, &branch_location, &inferred);
                }
                IrElseBranch::Else { body, .. } => {
                    // inferring block
                    let branch_location = body.get_location();
                    let inferred = self.infer_block(body)?;
                    expected = self.unify(&if_location, &expected, &branch_location, &inferred);
                }
            }
        }
        Some(expected)
    }

    /// Infers while
    fn infer_while(
        &mut self,
        location: Address,
        logical: IrExpression,
        body: IrBlock,
    ) -> Option<Typ> {
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
        let inferred = self.infer_block(body);
        // popping rib
        self.resolver.pop_rib();
        inferred
    }

    /// Analyzes define
    pub(crate) fn analyze_define(
        &mut self,
        location: Address,
        name: EcoString,
        value: IrExpression,
        typ: Option<TypePath>,
    ) {
        let inferred_value = self.infer_expr(value);
        match typ {
            Some(annotated_path) => {
                let annotated_location = annotated_path.get_location();
                let annotated = self.infer_type_annotation(annotated_path);
                self.unify(&location, &inferred_value, &annotated_location, &annotated);
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
    fn analyze_assignment(&mut self, location: Address, what: IrExpression, value: IrExpression) {
        let inferred_what = self.infer_expr(what);
        let value_location = value.get_location();
        let inferred_value = self.infer_expr(value);
        self.unify(&location, &inferred_what, &value_location, &inferred_value);
    }

    /// Analyzes call
    fn analyze_call(&mut self, location: Address, what: IrExpression, args: Vec<IrExpression>) {
        let _ = self.infer_call(location, what, args);
    }

    /// Analyzes funciton statement
    fn analyze_function(
        &mut self,
        location: Address,
        name: EcoString,
        params: Vec<IrParameter>,
        body: IrBlock,
        ret_type: Option<TypePath>,
    ) {
        // inferring return type
        let ret = ret_type.map_or(Typ::Void, |t| self.infer_type_annotation(t));

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
        let block_location = body.get_location();
        let inferred_block = self.infer_block(body).unwrap_or(Typ::Void);
        self.unify(&location, &ret, &block_location, &inferred_block);
        self.resolver.pop_rib();
    }

    /// Infers statement
    fn infer_stmt(&mut self, statement: IrStatement) -> Option<Typ> {
        match statement {
            IrStatement::If {
                location,
                logical,
                body,
                else_branches,
            } => self.infer_if(location, logical, body, else_branches),
            IrStatement::While {
                location,
                logical,
                body,
            } => self.infer_while(location, logical, body),
            IrStatement::Define {
                location,
                name,
                value,
                typ,
            } => {
                self.analyze_define(location, name, value, typ);
                None
            }
            IrStatement::Assign {
                location,
                what,
                value,
            } => {
                self.analyze_assignment(location, what, value);
                None
            }
            IrStatement::Call {
                location,
                what,
                args,
            } => {
                self.analyze_call(location, what, args);
                None
            }
            IrStatement::Fn {
                location,
                name,
                params,
                body,
                typ,
            } => {
                self.analyze_function(location, name, params, body, typ);
                None
            }
            IrStatement::Break { .. } => None,
            IrStatement::Continue { .. } => None,
            IrStatement::Return { value, .. } => Some(
                value
                    .map(|value| self.infer_expr(value))
                    .unwrap_or(Typ::Void),
            ),
            IrStatement::For { .. } => todo!(),
            IrStatement::Match {
                location,
                value,
                cases,
            } => self.infer_pattern_matching(location, value, cases),
        }
    }

    /// Infers block
    pub(crate) fn infer_block(&mut self, block: IrBlock) -> Option<Typ> {
        // Epxected type, used to
        // unify type of all block statements
        let mut expected = None;
        let mut location = block.get_location();
        // Inferring each statement
        // and unifying them.
        for stmt in block.statements {
            let stmt_location = stmt.get_location();
            match &expected {
                Some(expected_typ) => {
                    let inferred = match self.infer_stmt(stmt) {
                        Some(typ) => typ,
                        _ => continue,
                    };
                    expected = Some(self.unify(&location, expected_typ, &stmt_location, &inferred));
                }
                _ => {
                    location = stmt.get_location();
                    expected = Some(match self.infer_stmt(stmt) {
                        None => continue,
                        Some(typ) => typ,
                    });
                }
            }
        }
        expected
    }
}
