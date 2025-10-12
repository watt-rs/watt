/// Imports
use crate::{errors::CfaError, flow::Flow};
use miette::NamedSource;
use oil_common::{address::Address, bail};
use oil_ir::ir::{
    IrBlock, IrCase, IrDeclaration, IrElseBranch, IrExpression, IrModule, IrPattern, IrStatement,
};
use std::sync::Arc;

/// Module cfa cx
pub struct ModuleCfaCx<'src> {
    src: &'src NamedSource<Arc<String>>,
}

/// Implementation
impl<'src> ModuleCfaCx<'src> {
    /// Creates new cfa context
    pub fn new(src: &'src NamedSource<Arc<String>>) -> Self {
        return Self { src };
    }

    /// Checks for loop breaks,
    /// if detected raises error
    fn pass(&self, flow: Flow) -> Flow {
        match flow {
            Flow::Continue(address) => bail!(CfaError::ContinueWithoutLoop {
                src: self.src.clone(),
                span: address.span.clone().into()
            }),
            Flow::Break(address) => bail!(CfaError::BreakWithoutLoop {
                src: self.src.clone(),
                span: address.span.clone().into()
            }),
            it => it,
        }
    }

    /// Analyzes expression
    fn analyze_expr(&self, expr: &IrExpression) {
        match expr {
            IrExpression::Bin { left, right, .. } => {
                self.analyze_expr(left);
                self.analyze_expr(right);
            }
            IrExpression::Unary { value, .. } => {
                self.analyze_expr(value);
            }
            IrExpression::FieldAccess { container, .. } => {
                self.analyze_expr(container);
            }
            IrExpression::Call { what, args, .. } => {
                self.analyze_expr(what);
                for arg in args {
                    self.analyze_expr(arg);
                }
            }
            IrExpression::Range { from, to, .. } => {
                self.analyze_expr(from);
                self.analyze_expr(to);
            }
            IrExpression::AnFn { body, typ, .. } => {
                let flow = self.pass(self.analyze_block(&body));
                if typ.is_some() {
                    match flow {
                        Flow::Return(_) => {}
                        _ => bail!(CfaError::NotAllOfBranchesReturnsValue {
                            src: self.src.clone(),
                            span: body.get_location().span.into()
                        }),
                    }
                }
            }
            IrExpression::Match {
                location,
                value,
                cases,
            } => {
                self.analyze_match(location, value, cases);
            }
            _ => {}
        }
    }

    /// Analyzes match
    fn analyze_match(&self, location: &Address, value: &IrExpression, cases: &Vec<IrCase>) -> Flow {
        self.analyze_expr(value);
        // Checking defualt cases amount
        let defaults_amount = cases
            .iter()
            .filter(|case| {
                if let IrPattern::Default = case.pattern {
                    true
                } else {
                    false
                }
            })
            .count();
        if defaults_amount > 1 {
            bail!(CfaError::ManyDefaultCases {
                src: self.src.clone(),
                span: location.clone().span.into()
            })
        } else if defaults_amount < 1 {
            bail!(CfaError::NoDefaultCaseFound {
                src: self.src.clone(),
                span: location.clone().span.into()
            })
        }
        // checking cases
        let mut expected = None;
        for case in cases {
            // Analyzing branch flow
            let branch_flow = self.pass(self.analyze_block(&case.body));
            // Unifying flows
            expected = match (expected, branch_flow) {
                (Some(Flow::Return(l1)), Flow::Return(l2)) => Some(Flow::Return(l1 + l2)),
                (Some(a), b) => Some(Flow::Normal(a.get_location() + b.get_location())),
                (_, b) => Some(b),
            };
        }
        expected.unwrap_or(Flow::Normal(location.clone()))
    }

    /// Analyzes if
    fn analyze_if(
        &self,
        logical: &IrExpression,
        body: &IrBlock,
        else_branches: &Vec<IrElseBranch>,
    ) -> Flow {
        // Analyzing logical
        self.analyze_expr(logical);
        // Analyzing base if flow
        let mut flow = self.analyze_block(body);
        // Analyzing else branches
        let mut else_reached = false;
        for branch in else_branches {
            // Analyzing branch flow
            let branch_flow = match branch {
                IrElseBranch::Elif { logical, body, .. } => {
                    self.analyze_expr(logical);
                    self.analyze_block(body)
                }
                IrElseBranch::Else { body, .. } => {
                    else_reached = true;
                    self.analyze_block(body)
                }
            };
            // Unifying flows
            flow = match (flow, branch_flow) {
                (Flow::Return(l1), Flow::Return(l2)) => Flow::Return(l1 + l2),
                (a, b) => Flow::Normal(a.get_location() + b.get_location()),
            };
        }
        // Cheking for else case
        match &flow {
            Flow::Return(address) => {
                if else_reached {
                    flow
                } else {
                    Flow::Normal(address.clone())
                }
            }
            _ => flow,
        }
    }

    /// Analyzes while
    fn analyze_while(&self, location: &Address, logical: &IrExpression, body: &IrBlock) -> Flow {
        self.analyze_expr(logical);
        self.analyze_block(body);
        Flow::Normal(location.clone())
    }

    /// Analyzes statement
    fn analyze_stmt(&self, stmt: &IrStatement) -> Flow {
        match stmt {
            IrStatement::If {
                body,
                logical,
                else_branches,
                ..
            } => self.analyze_if(logical, body, else_branches),
            IrStatement::While {
                location,
                logical,
                body,
            } => self.analyze_while(location, logical, body),
            IrStatement::Define {
                location, value, ..
            } => {
                self.analyze_expr(value);
                Flow::Normal(location.clone())
            }
            IrStatement::Assign {
                location,
                what,
                value,
            } => {
                self.analyze_expr(what);
                self.analyze_expr(value);
                Flow::Normal(location.clone())
            }
            IrStatement::Call {
                location,
                what,
                args,
            } => {
                self.analyze_expr(what);
                for arg in args {
                    self.analyze_expr(arg);
                }
                Flow::Normal(location.clone())
            }
            IrStatement::Fn {
                location,
                body,
                typ,
                ..
            } => {
                let flow = self.pass(self.analyze_block(body));
                if typ.is_some() {
                    match flow {
                        Flow::Return(_) => Flow::Normal(location.clone()),
                        _ => bail!(CfaError::NotAllOfBranchesReturnsValue {
                            src: self.src.clone(),
                            span: body.get_location().span.into()
                        }),
                    }
                } else {
                    Flow::Normal(location.clone())
                }
            }
            IrStatement::Break { location } => Flow::Break(location.clone()),
            IrStatement::Continue { location } => Flow::Continue(location.clone()),
            IrStatement::Return { location, .. } => Flow::Return(location.clone()),
            IrStatement::For { .. } => todo!(),
            IrStatement::Match {
                location,
                value,
                cases,
            } => self.analyze_match(location, value, cases),
        }
    }

    /// Analyzes block
    fn analyze_block(&self, block: &IrBlock) -> Flow {
        for stmt in &block.statements {
            match self.analyze_stmt(&stmt) {
                Flow::Return(address) => return Flow::Return(address),
                Flow::Continue(address) => return Flow::Continue(address),
                Flow::Break(address) => return Flow::Break(address),
                Flow::Normal(_) => continue,
            }
        }
        Flow::Normal(block.get_location())
    }

    /// Analyzes declaration
    fn analyze_decl(&self, decl: &IrDeclaration) {
        match decl {
            IrDeclaration::Function(ir_function) => {
                let flow = self.pass(self.analyze_block(&ir_function.body));
                if ir_function.typ.is_some() {
                    match flow {
                        Flow::Return(_) => {}
                        _ => bail!(CfaError::NotAllOfBranchesReturnsValue {
                            src: self.src.clone(),
                            span: ir_function.body.get_location().span.into()
                        }),
                    }
                }
            }
            IrDeclaration::Variable(ir_variable) => {
                self.analyze_expr(&ir_variable.value);
            }
            IrDeclaration::Type(ir_type) => {
                for var in &ir_type.fields {
                    self.analyze_expr(&var.value);
                }
                for function in &ir_type.functions {
                    let flow = self.pass(self.analyze_block(&function.body));
                    if function.typ.is_some() {
                        match flow {
                            Flow::Return(_) => {}
                            _ => bail!(CfaError::NotAllOfBranchesReturnsValue {
                                src: self.src.clone(),
                                span: function.body.get_location().span.into()
                            }),
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// Analyzes module
    pub fn analyze(&self, ir: &IrModule) {
        for decl in &ir.definitions {
            self.analyze_decl(decl);
        }
    }
}
