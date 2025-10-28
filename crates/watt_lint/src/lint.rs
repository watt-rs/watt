/// Imports
use crate::{
    case::{self},
    consts,
    warnings::LintWarning,
};
use miette::NamedSource;
use std::sync::Arc;
use watt_ast::ast::{Block, Declaration, Either, ElseBranch, Expression, Module, Statement};
use watt_common::{package::DraftPackage, warn};

/// Linting context
pub struct LintCx<'cx, 'module> {
    /// Source
    pub source: &'cx NamedSource<Arc<String>>,
    /// Draft package
    pub draft: &'cx DraftPackage,
    /// Module
    pub module: &'module Module,
}

/// Implementation
impl<'cx, 'module> LintCx<'cx, 'module> {
    /// Creates new context
    pub fn new(
        source: &'cx NamedSource<Arc<String>>,
        draft: &'cx DraftPackage,
        module: &'module Module,
    ) -> Self {
        Self {
            source,
            draft,
            module,
        }
    }

    /// Lints module
    pub fn lint(&self) {
        for decl in &self.module.declarations {
            self.lint_decl(decl);
        }
    }

    /// Lints declaration
    fn lint_decl(&self, decl: &Declaration) {
        match decl {
            Declaration::TypeDeclaration {
                location,
                name,
                constructor,
                declarations,
                ..
            } => {
                // Checking type name is in `PascalCase`
                if !case::is_pascal_case(name) {
                    warn!(
                        self,
                        LintWarning::WrongTypeName {
                            src: self.source.clone(),
                            span: location.span.clone().into()
                        }
                    )
                }

                // Checking that constructor has < consts::MAX_PARAMS params.
                if constructor.len() > consts::MAX_PARAMS {
                    warn!(
                        self,
                        LintWarning::TooManyParams {
                            src: self.source.clone(),
                            span: location.span.clone().into(),
                            name: name.clone()
                        }
                    )
                }

                // Declarations
                for decl in declarations {
                    self.lint_decl(decl);
                }
            }
            Declaration::EnumDeclaration {
                location,
                name,
                variants,
                ..
            } => {
                // Checking type name is in `PascalCase`
                if !case::is_pascal_case(name) {
                    warn!(
                        self,
                        LintWarning::WrongTypeName {
                            src: self.source.clone(),
                            span: location.span.clone().into()
                        }
                    )
                }

                // Checking that variants count < consts::MAX_PARAMS params.
                if variants.len() > consts::MAX_PARAMS {
                    warn!(
                        self,
                        LintWarning::TooManyParams {
                            src: self.source.clone(),
                            span: location.span.clone().into(),
                            name: name.clone()
                        }
                    )
                }

                // Variants
                for variant in variants {
                    // Variant type name is in `PascalCase`
                    if !case::is_pascal_case(&variant.name) {
                        warn!(
                            self,
                            LintWarning::WrongVariantName {
                                src: self.source.clone(),
                                span: variant.location.span.clone().into()
                            }
                        )
                    }

                    // Checking that variant fields count < consts::MAX_PARAMS params.
                    if variant.params.len() > consts::MAX_PARAMS {
                        warn!(
                            self,
                            LintWarning::TooManyParams {
                                src: self.source.clone(),
                                span: variant.location.span.clone().into(),
                                name: name.clone()
                            }
                        )
                    }
                }
            }
            Declaration::TraitDeclaration {
                location,
                name,
                functions,
                ..
            } => {
                // Checking trait name is in `PascalCase`
                if !case::is_pascal_case(&name) {
                    warn!(
                        self,
                        LintWarning::WrongTypeName {
                            src: self.source.clone(),
                            span: location.span.clone().into()
                        }
                    )
                }

                // Linting functions
                for function in functions {
                    // Checking function name is in `snake_case`
                    if !case::is_snake_case(&function.name) {
                        warn!(
                            self,
                            LintWarning::WrongFunctionName {
                                src: self.source.clone(),
                                span: function.location.span.clone().into()
                            }
                        )
                    }
                    // Checking that function has < consts::MAX_PARAMS params.
                    if function.params.len() > consts::MAX_PARAMS {
                        warn!(
                            self,
                            LintWarning::TooManyParamsInAnFn {
                                src: self.source.clone(),
                                span: location.span.clone().into()
                            }
                        )
                    }
                }
            }
            Declaration::ExternFunction {
                location,
                name,
                params,
                ..
            } => {
                // Checking function name is in `snake_case`
                if !case::is_snake_case(&name) {
                    warn!(
                        self,
                        LintWarning::WrongVariantName {
                            src: self.source.clone(),
                            span: location.span.clone().into()
                        }
                    )
                }
                // Checking that function has < consts::MAX_PARAMS params.
                if params.len() > consts::MAX_PARAMS {
                    warn!(
                        self,
                        LintWarning::TooManyParamsInAnFn {
                            src: self.source.clone(),
                            span: location.span.clone().into()
                        }
                    )
                }
            }
            Declaration::VarDef {
                location,
                name,
                value,
                ..
            } => {
                // Checking variable name is in `snake_case`
                if !case::is_snake_case(&name) {
                    warn!(
                        self,
                        LintWarning::WrongVariantName {
                            src: self.source.clone(),
                            span: location.span.clone().into()
                        }
                    )
                }
                self.lint_expr(value);
            }
            Declaration::Function {
                location,
                name,
                params,
                body,
                ..
            } => {
                match body {
                    Either::Left(block) => self.lint_block(block),
                    Either::Right(expr) => self.lint_expr(expr),
                }
                // Checking function name is in `snake_case`
                if !case::is_snake_case(&name) {
                    warn!(
                        self,
                        LintWarning::WrongFunctionName {
                            src: self.source.clone(),
                            span: location.span.clone().into()
                        }
                    )
                }
                // Checking that function has < consts::MAX_PARAMS params.
                if params.len() > consts::MAX_PARAMS {
                    warn!(
                        self,
                        LintWarning::TooManyParamsInAnFn {
                            src: self.source.clone(),
                            span: location.span.clone().into()
                        }
                    )
                }
            }
        }
    }

    /// Lints block
    fn lint_block(&self, block: &Block) {
        // Checking that block has at least 1 statement
        if block.body.len() < 1 {
            warn!(
                self,
                LintWarning::EmptyBlock {
                    src: self.source.clone(),
                    span: block.location.span.clone().into()
                }
            );
            return;
        }
        // Linting statements
        for stmt in &block.body {
            self.lint_statement(stmt);
        }
    }

    /// Lints statement
    fn lint_statement(&self, stmt: &Statement) {
        match stmt {
            Statement::VarDef {
                location,
                name,
                value,
                ..
            } => {
                // Checking variable is in `snake_case`
                if !case::is_snake_case(&name) {
                    warn!(
                        self,
                        LintWarning::WrongVariableName {
                            src: self.source.clone(),
                            span: location.span.clone().into(),
                        }
                    )
                }
                self.lint_expr(value);
            }
            Statement::VarAssign { what, value, .. } => {
                self.lint_expr(what);
                self.lint_expr(value);
            }
            Statement::Expr(expr) => {
                self.lint_expr(expr);
            }
            Statement::Loop { logical, body, .. } => {
                self.lint_expr(logical);
                match body {
                    Either::Left(block) => self.lint_block(block),
                    Either::Right(expr) => self.lint_expr(expr),
                }
            }
            Statement::Semi(expr) => {
                self.lint_expr(expr);
            }
        }
    }

    /// Lints expression
    fn lint_expr(&self, expr: &Expression) {
        match expr {
            Expression::Bin { left, right, .. } => {
                self.lint_expr(left);
                self.lint_expr(right);
            }
            Expression::Unary { value, .. } => {
                self.lint_expr(value);
            }
            Expression::If {
                logical,
                body,
                else_branches,
                ..
            } => {
                // Linting current branch
                self.lint_expr(logical);
                match body {
                    Either::Left(block) => self.lint_block(block),
                    Either::Right(expr) => self.lint_expr(expr),
                }

                // Linting else branches
                for branch in else_branches {
                    match branch {
                        ElseBranch::Elif { logical, body, .. } => {
                            self.lint_expr(logical);
                            match body {
                                Either::Left(block) => self.lint_block(block),
                                Either::Right(expr) => self.lint_expr(expr),
                            }
                        }
                        ElseBranch::Else { body, .. } => match body {
                            Either::Left(block) => self.lint_block(block),
                            Either::Right(expr) => self.lint_expr(expr),
                        },
                    }
                }
            }
            Expression::SuffixVar { container, .. } => {
                self.lint_expr(container);
            }
            Expression::Call { what, args, .. } => {
                self.lint_expr(what);
                for arg in args {
                    self.lint_expr(arg);
                }
            }
            Expression::Function {
                location,
                params,
                body,
                ..
            } => {
                // Linting body
                match body {
                    Either::Left(block) => self.lint_block(block),
                    Either::Right(expr) => self.lint_expr(expr),
                }
                // Checking that function has < consts::MAX_PARAMS params.
                if params.len() > consts::MAX_PARAMS {
                    warn!(
                        self,
                        LintWarning::TooManyParamsInAnFn {
                            src: self.source.clone(),
                            span: location.span.clone().into()
                        }
                    )
                }
            }
            Expression::Match { value, cases, .. } => {
                self.lint_expr(value);
                for case in cases {
                    match &case.body {
                        Either::Left(block) => self.lint_block(block),
                        Either::Right(expr) => self.lint_expr(expr),
                    }
                }
            }
            _ => {}
        }
    }
}
