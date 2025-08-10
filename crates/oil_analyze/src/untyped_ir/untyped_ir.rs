use ecow::EcoString;
/// Imports
use oil_ast::ast::{DependencyPath, Publicity, TypePath};
use oil_common::address::Address;

/// Ir Parameter
#[derive(Debug)]
pub struct UntypedIrParameter {
    pub name: EcoString,
    pub typ: TypePath,
}

/// Untyped statement
#[derive(Debug)]
pub enum UntypedStatement {
    If {
        location: Address,
        logical: UntypedExpression,
        body: UntypedBlock,
        elseif: Option<Box<UntypedStatement>>,
    },
    While {
        location: Address,
        logical: UntypedExpression,
        body: UntypedBlock,
    },
    Define {
        location: Address,
        name: EcoString,
        value: UntypedExpression,
        typ: Option<TypePath>,
    },
    Assign {
        location: Address,
        base: Option<UntypedExpression>,
        name: EcoString,
        value: UntypedExpression,
    },
    Get {
        location: Address,
        base: Option<UntypedExpression>,
        name: UntypedExpression,
    },
    Call {
        location: Address,
        base: Option<UntypedExpression>,
        name: EcoString,
        args: Vec<UntypedExpression>,
    },
    Fn {
        location: Address,
        name: EcoString,
        params: Vec<UntypedIrParameter>,
        body: UntypedBlock,
        typ: Option<TypePath>,
    },
    Break {
        location: Address,
    },
    Continue {
        location: Address,
    },
    For {
        location: Address,
        iterable: UntypedExpression,
        variable: EcoString,
        body: UntypedBlock,
    },
    Return {
        location: Address,
        value: UntypedExpression,
    },
}

/// Untyped expression
#[derive(Debug)]
pub enum UntypedExpression {
    Float {
        location: Address,
        value: f64,
    },
    Int {
        location: Address,
        value: i64,
    },
    String {
        location: Address,
        value: EcoString,
    },
    Bool {
        location: Address,
        value: EcoString,
    },
    Bin {
        location: Address,
        left: Box<UntypedExpression>,
        right: Box<UntypedExpression>,
        op: BinaryOperator,
    },
    Unary {
        location: Address,
        value: Box<UntypedExpression>,
        op: UnaryOperator,
    },
    Get {
        location: Address,
        base: Option<Box<UntypedExpression>>,
        name: EcoString,
    },
    Call {
        location: Address,
        base: Option<Box<UntypedExpression>>,
        name: EcoString,
        args: Vec<UntypedExpression>,
    },
    Range {
        location: Address,
        from: Box<UntypedExpression>,
        to: Box<UntypedExpression>,
    },
}

/// Binary operator
#[derive(Debug)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Or,
    And,
    Xor,
    BitwiseAnd,
    BitwiseOr,
    Mod,
    Eq,
    Neq,
    Gt,
    Lt,
    Ge,
    Le,
}

/// Unary operator
#[derive(Debug)]
pub enum UnaryOperator {
    Negate,
    Bang,
}

/// Untyped block
#[derive(Debug)]
pub struct UntypedBlock {
    pub nodes: Vec<UntypedStatement>,
}

/// Untyped function
#[derive(Debug)]
pub struct UntypedFunction {
    pub location: Address,
    pub name: EcoString,
    pub publicity: Publicity,
    pub params: Vec<UntypedIrParameter>,
    pub body: UntypedBlock,
    pub typ: Option<TypePath>,
}

/// Untyped variable
#[derive(Debug)]
pub struct UntypedVariable {
    pub location: Address,
    pub name: EcoString,
    pub publicity: Publicity,
    pub typ: Option<TypePath>,
    pub value: UntypedExpression,
}

/// Untyped type
#[derive(Debug)]
pub struct UntypedType {
    pub location: Address,
    pub name: EcoString,
    pub publicity: Publicity,
    pub constructor: Vec<UntypedIrParameter>,
    pub fields: Vec<UntypedVariable>,
    pub functions: Vec<UntypedFunction>,
}

/// Untyped declaration
#[derive(Debug)]
pub enum UntypedDeclaration {
    Function(UntypedFunction),
    Variable(UntypedVariable),
    Type(UntypedType),
}

/// Dependency
#[derive(Debug)]
pub struct Dependency {
    pub location: Address,
    pub name: Option<EcoString>,
    pub path: DependencyPath,
}

/// Untyped module
#[derive(Debug)]
pub struct UntypedModule {
    pub definitions: Vec<UntypedDeclaration>,
    pub dependencies: Vec<Dependency>,
}
