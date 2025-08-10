/// Imports
use crate::{
    errors::UntypedIrError,
    untyped_ir::{
        BinaryOperator, Dependency, IrParameter, UnaryOperator, UntypedBlock, UntypedDeclaration,
        UntypedExpression, UntypedFunction, UntypedModule, UntypedStatement, UntypedType,
        UntypedVariable,
    },
};
use miette::NamedSource;
use oil_ast::ast::{Node, Tree};
use oil_common::bail;

/// Node to ir declaration
pub fn node_to_ir_declaration(source: &NamedSource<String>, node: Node) -> UntypedDeclaration {
    match node {
        Node::Define {
            publicity,
            name,
            value,
            typ,
        } => UntypedDeclaration::Variable(UntypedVariable {
            location: name.address,
            publicity,
            name: name.value,
            value: node_to_ir_expression(source, *value),
            typ,
        }),
        Node::FnDeclaration {
            name,
            publicity,
            params,
            body,
            typ,
        } => UntypedDeclaration::Function(UntypedFunction {
            location: name.address,
            publicity,
            name: name.value,
            params: params
                .into_iter()
                .map(|param| IrParameter {
                    name: param.name.value,
                    typ: param.typ,
                })
                .collect(),
            body: node_to_ir_block(source, *body),
            typ,
        }),
        Node::TypeDeclaration {
            name,
            publicity,
            constructor,
            fields,
            functions,
        } => UntypedDeclaration::Type(UntypedType {
            location: name.address,
            publicity,
            name: name.value,
            constructor: constructor
                .into_iter()
                .map(|param| IrParameter {
                    name: param.name.value,
                    typ: param.typ,
                })
                .collect(),
            fields: fields
                .into_iter()
                .map(|n| match n {
                    Node::Define {
                        publicity,
                        name,
                        value,
                        typ,
                    } => UntypedVariable {
                        location: name.address,
                        name: name.value,
                        publicity,
                        typ,
                        value: node_to_ir_expression(source, *value),
                    },
                    unexpected => bail!(UntypedIrError::UnexpectedNodeInTypebody { unexpected }),
                })
                .collect(),
            functions: functions
                .into_iter()
                .map(|n| match n {
                    Node::FnDeclaration {
                        publicity,
                        name,
                        typ,
                        params,
                        body,
                    } => UntypedFunction {
                        location: name.address,
                        name: name.value,
                        publicity,
                        typ,
                        params: params
                            .into_iter()
                            .map(|param| IrParameter {
                                name: param.name.value,
                                typ: param.typ,
                            })
                            .collect(),
                        body: node_to_ir_block(source, *body),
                    },
                    unexpected => bail!(UntypedIrError::UnexpectedNodeInTypebody { unexpected }),
                })
                .collect(),
        }),
        unexpected => bail!(UntypedIrError::UnexpectedDeclarationNode { unexpected }),
    }
}

/// Node to ir block
pub fn node_to_ir_block(source: &NamedSource<String>, node: Node) -> UntypedBlock {
    match node {
        Node::Block { body } => UntypedBlock {
            nodes: body
                .into_iter()
                .map(|n| node_to_ir_statement(source, n))
                .collect(),
        },
        unexpected => bail!(UntypedIrError::UnexpectedBlockNode { unexpected }),
    }
}

/// Node to ir expression
pub fn node_to_ir_expression(source: &NamedSource<String>, node: Node) -> UntypedExpression {
    match node {
        Node::Number { value } => {
            if value.value.contains('.') {
                match str::parse::<f64>(&value.value) {
                    Ok(ok) => UntypedExpression::Float {
                        location: value.address,
                        value: ok,
                    },
                    Err(_) => bail!(UntypedIrError::FailedToParseF64 {
                        src: source.clone(),
                        span: value.address.span.into()
                    }),
                }
            } else {
                match str::parse::<i64>(&value.value) {
                    Ok(ok) => UntypedExpression::Int {
                        location: value.address,
                        value: ok,
                    },
                    Err(_) => bail!(UntypedIrError::FailedToParseI64 {
                        src: source.clone(),
                        span: value.address.span.into()
                    }),
                }
            }
        }
        Node::String { value } => UntypedExpression::String {
            location: value.address,
            value: value.value,
        },
        Node::Bool { value } => UntypedExpression::Bool {
            location: value.address,
            value: value.value,
        },
        Node::Bin { left, right, op } => match op.value.as_str() {
            "+" => UntypedExpression::Bin {
                location: op.address,
                left: Box::new(node_to_ir_expression(source, *left)),
                right: Box::new(node_to_ir_expression(source, *right)),
                op: BinaryOperator::Add,
            },
            "-" => UntypedExpression::Bin {
                location: op.address,
                left: Box::new(node_to_ir_expression(source, *left)),
                right: Box::new(node_to_ir_expression(source, *right)),
                op: BinaryOperator::Sub,
            },
            "*" => UntypedExpression::Bin {
                location: op.address,
                left: Box::new(node_to_ir_expression(source, *left)),
                right: Box::new(node_to_ir_expression(source, *right)),
                op: BinaryOperator::Mul,
            },
            "/" => UntypedExpression::Bin {
                location: op.address,
                left: Box::new(node_to_ir_expression(source, *left)),
                right: Box::new(node_to_ir_expression(source, *right)),
                op: BinaryOperator::Div,
            },
            "%" => UntypedExpression::Bin {
                location: op.address,
                left: Box::new(node_to_ir_expression(source, *left)),
                right: Box::new(node_to_ir_expression(source, *right)),
                op: BinaryOperator::Div,
            },
            _ => bail!(UntypedIrError::UnknownOp {
                src: source.clone(),
                span: op.address.span.into()
            }),
        },
        Node::Unary { value, op } => match op.value.as_str() {
            "-" => UntypedExpression::Unary {
                location: op.address.clone(),
                value: Box::new(node_to_ir_expression(source, *value)),
                op: UnaryOperator::Negate,
            },
            "!" => UntypedExpression::Unary {
                location: op.address.clone(),
                value: Box::new(node_to_ir_expression(source, *value)),
                op: UnaryOperator::Bang,
            },
            _ => bail!(UntypedIrError::UnknownOp {
                src: source.clone(),
                span: op.address.span.into()
            }),
        },
        Node::Get { previous, name } => match previous {
            Some(some) => UntypedExpression::Get {
                location: name.address.clone(),
                base: Some(Box::new(node_to_ir_expression(source, *some))),
                name: name.value,
            },
            None => UntypedExpression::Get {
                location: name.address.clone(),
                base: None,
                name: name.value,
            },
        },
        Node::Call {
            previous,
            name,
            args,
        } => match previous {
            Some(base) => UntypedExpression::Call {
                base: Some(Box::new(node_to_ir_expression(source, *base))),
                location: name.address,
                name: name.value,
                args: args
                    .into_iter()
                    .map(|arg| node_to_ir_expression(source, arg))
                    .collect(),
            },
            None => UntypedExpression::Call {
                base: None,
                location: name.address,
                name: name.value,
                args: args
                    .into_iter()
                    .map(|arg| node_to_ir_expression(source, arg))
                    .collect(),
            },
        },
        Node::Cond { left, right, op } => match op.value.as_str() {
            ">" => UntypedExpression::Bin {
                location: op.address,
                left: Box::new(node_to_ir_expression(source, *left)),
                right: Box::new(node_to_ir_expression(source, *right)),
                op: BinaryOperator::Gt,
            },
            "<" => UntypedExpression::Bin {
                location: op.address,
                left: Box::new(node_to_ir_expression(source, *left)),
                right: Box::new(node_to_ir_expression(source, *right)),
                op: BinaryOperator::Lt,
            },
            "==" => UntypedExpression::Bin {
                location: op.address,
                left: Box::new(node_to_ir_expression(source, *left)),
                right: Box::new(node_to_ir_expression(source, *right)),
                op: BinaryOperator::Eq,
            },
            "!=" => UntypedExpression::Bin {
                location: op.address,
                left: Box::new(node_to_ir_expression(source, *left)),
                right: Box::new(node_to_ir_expression(source, *right)),
                op: BinaryOperator::Neq,
            },
            ">=" => UntypedExpression::Bin {
                location: op.address,
                left: Box::new(node_to_ir_expression(source, *left)),
                right: Box::new(node_to_ir_expression(source, *right)),
                op: BinaryOperator::Ge,
            },
            "<=" => UntypedExpression::Bin {
                location: op.address,
                left: Box::new(node_to_ir_expression(source, *left)),
                right: Box::new(node_to_ir_expression(source, *right)),
                op: BinaryOperator::Le,
            },
            _ => bail!(UntypedIrError::UnknownOp {
                src: source.clone(),
                span: op.address.span.into()
            }),
        },
        Node::Logical { left, right, op } => match op.value.as_str() {
            "&&" => UntypedExpression::Bin {
                location: op.address,
                left: Box::new(node_to_ir_expression(source, *left)),
                right: Box::new(node_to_ir_expression(source, *right)),
                op: BinaryOperator::And,
            },
            "||" => UntypedExpression::Bin {
                location: op.address,
                left: Box::new(node_to_ir_expression(source, *left)),
                right: Box::new(node_to_ir_expression(source, *right)),
                op: BinaryOperator::Or,
            },
            _ => bail!(UntypedIrError::UnknownOp {
                src: source.clone(),
                span: op.address.span.into()
            }),
        },
        Node::Range { location, from, to } => UntypedExpression::Range {
            location: location.address,
            from: Box::new(node_to_ir_expression(source, *from)),
            to: Box::new(node_to_ir_expression(source, *to)),
        },
        unexpected => bail!(UntypedIrError::UnexpectedExpressionNode { unexpected }),
    }
}

/// Node to ir statement
pub fn node_to_ir_statement(source: &NamedSource<String>, node: Node) -> UntypedStatement {
    match node {
        Node::If {
            location,
            logical,
            body,
            elseif,
        } => match elseif {
            Some(elseif) => UntypedStatement::If {
                location: location.address,
                logical: node_to_ir_expression(source, *logical),
                body: node_to_ir_block(source, *body),
                elseif: Some(Box::new(node_to_ir_statement(source, *elseif))),
            },
            None => UntypedStatement::If {
                location: location.address,
                logical: node_to_ir_expression(source, *logical),
                body: node_to_ir_block(source, *body),
                elseif: None,
            },
        },
        Node::While {
            location,
            logical,
            body,
        } => UntypedStatement::While {
            location: location.address,
            logical: node_to_ir_expression(source, *logical),
            body: node_to_ir_block(source, *body),
        },
        Node::Define {
            name, value, typ, ..
        } => UntypedStatement::Define {
            location: name.address,
            name: name.value,
            value: node_to_ir_expression(source, *value),
            typ,
        },
        Node::Assign {
            previous,
            name,
            value,
        } => match previous {
            Some(base) => UntypedStatement::Assign {
                base: Some(node_to_ir_expression(source, *base)),
                location: name.address,
                name: name.value,
                value: node_to_ir_expression(source, *value),
            },
            None => UntypedStatement::Assign {
                base: None,
                location: name.address,
                name: name.value,
                value: node_to_ir_expression(source, *value),
            },
        },
        Node::Call {
            previous,
            name,
            args,
        } => match previous {
            Some(base) => UntypedStatement::Call {
                base: Some(node_to_ir_expression(source, *base)),
                location: name.address,
                name: name.value,
                args: args
                    .into_iter()
                    .map(|arg| node_to_ir_expression(source, arg))
                    .collect(),
            },
            None => UntypedStatement::Call {
                base: None,
                location: name.address,
                name: name.value,
                args: args
                    .into_iter()
                    .map(|arg| node_to_ir_expression(source, arg))
                    .collect(),
            },
        },
        Node::FnDeclaration {
            name,
            params,
            body,
            typ,
            ..
        } => UntypedStatement::Fn {
            location: name.address,
            name: name.value,
            params: params
                .into_iter()
                .map(|param| IrParameter {
                    name: param.name.value,
                    typ: param.typ,
                })
                .collect(),
            body: node_to_ir_block(source, *body),
            typ,
        },
        Node::Break { location } => UntypedStatement::Break {
            location: location.address,
        },
        Node::Continue { location } => UntypedStatement::Continue {
            location: location.address,
        },
        Node::Return { location, value } => UntypedStatement::Return {
            location: location.address,
            value: node_to_ir_expression(source, *value),
        },
        Node::For {
            iterable,
            variable,
            body,
        } => UntypedStatement::For {
            location: variable.address,
            iterable: node_to_ir_expression(source, *iterable),
            variable: variable.value,
            body: node_to_ir_block(source, *body),
        },
        unexpected => bail!(UntypedIrError::UnexpectedStatementNode { unexpected }),
    }
}

/// Tree to ir
pub fn tree_to_ir(source: &NamedSource<String>, tree: Tree) -> UntypedModule {
    let mut module = UntypedModule {
        dependencies: vec![],
        definitions: vec![],
    };

    // use and declaration
    for node in tree.body {
        match node {
            Node::Use {
                location,
                path,
                name,
            } => module.dependencies.push(Dependency {
                location,
                name: name.map(|n| n.value),
                path,
            }),
            declaration => module
                .definitions
                .push(node_to_ir_declaration(source, declaration)),
        };
    }

    module
}
