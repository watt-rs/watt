/// Imports
use crate::{
    errors::IrError,
    ir::{
        IrBinaryOp, IrBlock, IrCase, IrDeclaration, IrDependency, IrEnum, IrEnumConstructor,
        IrExpression, IrFunction, IrModule, IrParameter, IrPattern, IrStatement, IrType, IrUnaryOp,
        IrVariable,
    },
};
use miette::NamedSource;
use oil_ast::ast::{Node, Pattern, Tree};
use oil_common::bail;
use std::sync::Arc;

/// Node to ir declaration
pub fn node_to_ir_declaration(source: &NamedSource<Arc<String>>, node: Node) -> IrDeclaration {
    match node {
        Node::Define {
            publicity,
            name,
            value,
            typ,
        } => IrDeclaration::Variable(IrVariable {
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
        } => IrDeclaration::Function(IrFunction {
            location: name.address,
            publicity,
            name: name.value,
            params: params
                .into_iter()
                .map(|param| IrParameter {
                    location: param.name.address,
                    name: param.name.value,
                    typ: param.typ,
                })
                .collect(),
            body: node_to_ir_block(source, *body),
            typ,
        }),
        Node::TypeDeclaration {
            location,
            name,
            publicity,
            constructor,
            fields,
            functions,
        } => IrDeclaration::Type(IrType {
            location,
            publicity,
            name: name.value,
            constructor: constructor
                .into_iter()
                .map(|param| IrParameter {
                    location: param.name.address,
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
                    } => IrVariable {
                        location: name.address,
                        name: name.value,
                        publicity,
                        typ,
                        value: node_to_ir_expression(source, *value),
                    },
                    unexpected => bail!(IrError::UnexpectedNodeInTypebody { unexpected }),
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
                    } => IrFunction {
                        location: name.address,
                        name: name.value,
                        publicity,
                        typ,
                        params: params
                            .into_iter()
                            .map(|param| IrParameter {
                                location: param.name.address,
                                name: param.name.value,
                                typ: param.typ,
                            })
                            .collect(),
                        body: node_to_ir_block(source, *body),
                    },
                    unexpected => bail!(IrError::UnexpectedNodeInTypebody { unexpected }),
                })
                .collect(),
        }),
        Node::EnumDeclaration {
            location,
            name,
            publicity,
            variants,
        } => IrDeclaration::Enum(IrEnum {
            location,
            publicity,
            name: name.value,
            variants: variants
                .into_iter()
                .map(|v| IrEnumConstructor {
                    location: v.name.address,
                    name: v.name.value,
                    params: v
                        .params
                        .into_iter()
                        .map(|param| IrParameter {
                            location: param.name.address,
                            name: param.name.value,
                            typ: param.typ,
                        })
                        .collect(),
                })
                .collect(),
        }),
        unexpected => bail!(IrError::UnexpectedDeclarationNode { unexpected }),
    }
}

/// Node to ir block
pub fn node_to_ir_block(source: &NamedSource<Arc<String>>, node: Node) -> IrBlock {
    match node {
        Node::Block { body } => IrBlock {
            nodes: body
                .into_iter()
                .map(|n| node_to_ir_statement(source, n))
                .collect(),
        },
        unexpected => bail!(IrError::UnexpectedBlockNode { unexpected }),
    }
}

/// Node to ir expression
pub fn node_to_ir_expression(source: &NamedSource<Arc<String>>, node: Node) -> IrExpression {
    match node {
        Node::Number { value } => {
            if value.value.contains('.') {
                match str::parse::<f64>(&value.value) {
                    Ok(ok) => IrExpression::Float {
                        location: value.address,
                        value: ok,
                    },
                    Err(_) => bail!(IrError::FailedToParseF64 {
                        src: source.clone(),
                        span: value.address.span.into()
                    }),
                }
            } else {
                match str::parse::<i64>(&value.value) {
                    Ok(ok) => IrExpression::Int {
                        location: value.address,
                        value: ok,
                    },
                    Err(_) => bail!(IrError::FailedToParseI64 {
                        src: source.clone(),
                        span: value.address.span.into()
                    }),
                }
            }
        }
        Node::String { value } => IrExpression::String {
            location: value.address,
            value: value.value,
        },
        Node::Bool { value } => IrExpression::Bool {
            location: value.address,
            value: value.value,
        },
        Node::Bin { left, right, op } => match op.value.as_str() {
            "+" => IrExpression::Bin {
                location: op.address,
                left: Box::new(node_to_ir_expression(source, *left)),
                right: Box::new(node_to_ir_expression(source, *right)),
                op: IrBinaryOp::Add,
            },
            "-" => IrExpression::Bin {
                location: op.address,
                left: Box::new(node_to_ir_expression(source, *left)),
                right: Box::new(node_to_ir_expression(source, *right)),
                op: IrBinaryOp::Sub,
            },
            "*" => IrExpression::Bin {
                location: op.address,
                left: Box::new(node_to_ir_expression(source, *left)),
                right: Box::new(node_to_ir_expression(source, *right)),
                op: IrBinaryOp::Mul,
            },
            "/" => IrExpression::Bin {
                location: op.address,
                left: Box::new(node_to_ir_expression(source, *left)),
                right: Box::new(node_to_ir_expression(source, *right)),
                op: IrBinaryOp::Div,
            },
            "%" => IrExpression::Bin {
                location: op.address,
                left: Box::new(node_to_ir_expression(source, *left)),
                right: Box::new(node_to_ir_expression(source, *right)),
                op: IrBinaryOp::Div,
            },
            _ => bail!(IrError::UnknownOp {
                src: source.clone(),
                span: op.address.span.into()
            }),
        },
        Node::Unary { value, op } => match op.value.as_str() {
            "-" => IrExpression::Unary {
                location: op.address.clone(),
                value: Box::new(node_to_ir_expression(source, *value)),
                op: IrUnaryOp::Negate,
            },
            "!" => IrExpression::Unary {
                location: op.address.clone(),
                value: Box::new(node_to_ir_expression(source, *value)),
                op: IrUnaryOp::Bang,
            },
            _ => bail!(IrError::UnknownOp {
                src: source.clone(),
                span: op.address.span.into()
            }),
        },
        Node::Get { name } => IrExpression::Get {
            location: name.address,
            name: name.value,
        },
        Node::FieldAccess { container, name } => IrExpression::FieldAccess {
            location: name.address,
            container: Box::new(node_to_ir_expression(source, *container)),
            name: name.value,
        },
        Node::Call {
            location,
            what,
            args,
        } => IrExpression::Call {
            location,
            what: Box::new(node_to_ir_expression(source, *what)),
            args: args
                .into_iter()
                .map(|arg| node_to_ir_expression(source, arg))
                .collect(),
        },
        Node::Cond { left, right, op } => match op.value.as_str() {
            ">" => IrExpression::Bin {
                location: op.address,
                left: Box::new(node_to_ir_expression(source, *left)),
                right: Box::new(node_to_ir_expression(source, *right)),
                op: IrBinaryOp::Gt,
            },
            "<" => IrExpression::Bin {
                location: op.address,
                left: Box::new(node_to_ir_expression(source, *left)),
                right: Box::new(node_to_ir_expression(source, *right)),
                op: IrBinaryOp::Lt,
            },
            "==" => IrExpression::Bin {
                location: op.address,
                left: Box::new(node_to_ir_expression(source, *left)),
                right: Box::new(node_to_ir_expression(source, *right)),
                op: IrBinaryOp::Eq,
            },
            "!=" => IrExpression::Bin {
                location: op.address,
                left: Box::new(node_to_ir_expression(source, *left)),
                right: Box::new(node_to_ir_expression(source, *right)),
                op: IrBinaryOp::Neq,
            },
            ">=" => IrExpression::Bin {
                location: op.address,
                left: Box::new(node_to_ir_expression(source, *left)),
                right: Box::new(node_to_ir_expression(source, *right)),
                op: IrBinaryOp::Ge,
            },
            "<=" => IrExpression::Bin {
                location: op.address,
                left: Box::new(node_to_ir_expression(source, *left)),
                right: Box::new(node_to_ir_expression(source, *right)),
                op: IrBinaryOp::Le,
            },
            _ => bail!(IrError::UnknownOp {
                src: source.clone(),
                span: op.address.span.into()
            }),
        },
        Node::Logical { left, right, op } => match op.value.as_str() {
            "&&" => IrExpression::Bin {
                location: op.address,
                left: Box::new(node_to_ir_expression(source, *left)),
                right: Box::new(node_to_ir_expression(source, *right)),
                op: IrBinaryOp::And,
            },
            "||" => IrExpression::Bin {
                location: op.address,
                left: Box::new(node_to_ir_expression(source, *left)),
                right: Box::new(node_to_ir_expression(source, *right)),
                op: IrBinaryOp::Or,
            },
            _ => bail!(IrError::UnknownOp {
                src: source.clone(),
                span: op.address.span.into()
            }),
        },
        Node::Range { location, from, to } => IrExpression::Range {
            location,
            from: Box::new(node_to_ir_expression(source, *from)),
            to: Box::new(node_to_ir_expression(source, *to)),
        },
        Node::Match {
            location,
            value,
            cases,
        } => IrExpression::Match {
            location,
            value: Box::new(node_to_ir_expression(source, *value)),
            cases: cases
                .into_iter()
                .map(|case| IrCase {
                    location: case.address,
                    pattern: match case.pattern {
                        Pattern::Unwrap { en, fields } => IrPattern::Unwrap {
                            en: node_to_ir_expression(source, en),
                            fields: fields.into_iter().map(|field| field.value).collect(),
                        },
                        Pattern::Value(value) => {
                            IrPattern::Value(node_to_ir_expression(source, value))
                        }
                        Pattern::Range { start, end } => IrPattern::Range {
                            start: node_to_ir_expression(source, start),
                            end: node_to_ir_expression(source, end),
                        },
                    },
                    body: node_to_ir_block(source, case.body),
                })
                .collect(),
        },
        unexpected => bail!(IrError::UnexpectedExpressionNode { unexpected }),
    }
}

/// Node to ir statement
pub fn node_to_ir_statement(source: &NamedSource<Arc<String>>, node: Node) -> IrStatement {
    match node {
        Node::If {
            location,
            logical,
            body,
            elseif,
        } => match elseif {
            Some(elseif) => IrStatement::If {
                location: location,
                logical: node_to_ir_expression(source, *logical),
                body: node_to_ir_block(source, *body),
                elseif: Some(Box::new(node_to_ir_statement(source, *elseif))),
            },
            None => IrStatement::If {
                location: location,
                logical: node_to_ir_expression(source, *logical),
                body: node_to_ir_block(source, *body),
                elseif: None,
            },
        },
        Node::While {
            location,
            logical,
            body,
        } => IrStatement::While {
            location: location,
            logical: node_to_ir_expression(source, *logical),
            body: node_to_ir_block(source, *body),
        },
        Node::Define {
            name, value, typ, ..
        } => IrStatement::Define {
            location: name.address,
            name: name.value,
            value: node_to_ir_expression(source, *value),
            typ,
        },
        Node::Assign {
            location,
            what,
            value,
        } => IrStatement::Assign {
            location,
            what: Box::new(node_to_ir_expression(source, *what)),
            value: Box::new(node_to_ir_expression(source, *value)),
        },
        Node::Call {
            location,
            what,
            args,
        } => IrStatement::Call {
            location,
            what: Box::new(node_to_ir_expression(source, *what)),
            args: args
                .into_iter()
                .map(|arg| node_to_ir_expression(source, arg))
                .collect(),
        },
        Node::FnDeclaration {
            name,
            params,
            body,
            typ,
            ..
        } => IrStatement::Fn {
            location: name.address,
            name: name.value,
            params: params
                .into_iter()
                .map(|param| IrParameter {
                    location: param.name.address,
                    name: param.name.value,
                    typ: param.typ,
                })
                .collect(),
            body: node_to_ir_block(source, *body),
            typ,
        },
        Node::Break { location } => IrStatement::Break { location: location },
        Node::Continue { location } => IrStatement::Continue { location: location },
        Node::Return { location, value } => IrStatement::Return {
            location: location,
            value: node_to_ir_expression(source, *value),
        },
        Node::For {
            iterable,
            variable,
            body,
        } => IrStatement::For {
            location: variable.address,
            iterable: node_to_ir_expression(source, *iterable),
            variable: variable.value,
            body: node_to_ir_block(source, *body),
        },
        Node::Match {
            location,
            value,
            cases,
        } => IrStatement::Match {
            location: location,
            value: node_to_ir_expression(source, *value),
            cases: cases
                .into_iter()
                .map(|case| IrCase {
                    location: case.address,
                    pattern: match case.pattern {
                        Pattern::Unwrap { en, fields } => IrPattern::Unwrap {
                            en: node_to_ir_expression(source, en),
                            fields: fields.into_iter().map(|field| field.value).collect(),
                        },
                        Pattern::Value(value) => {
                            IrPattern::Value(node_to_ir_expression(source, value))
                        }
                        Pattern::Range { start, end } => IrPattern::Range {
                            start: node_to_ir_expression(source, start),
                            end: node_to_ir_expression(source, end),
                        },
                    },
                    body: node_to_ir_block(source, case.body),
                })
                .collect(),
        },
        unexpected => bail!(IrError::UnexpectedStatementNode { unexpected }),
    }
}

/// Tree to ir
pub fn tree_to_ir(source: NamedSource<Arc<String>>, tree: Tree) -> IrModule {
    let mut module = IrModule {
        dependencies: vec![],
        definitions: vec![],
        source: source.clone(),
    };

    // use and declaration
    for node in tree.body {
        match node {
            Node::Use {
                location,
                path,
                name,
            } => module.dependencies.push(IrDependency {
                location,
                name: name.map(|n| n.value),
                path: path.module,
            }),
            declaration => module
                .definitions
                .push(node_to_ir_declaration(&source, declaration)),
        };
    }

    module
}
