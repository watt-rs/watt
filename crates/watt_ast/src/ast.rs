/// Imports
use ecow::EcoString;
use miette::NamedSource;
use std::sync::Arc;
use watt_common::address::Address;

/// Dependency path
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DependencyPath {
    pub address: Address,
    pub module: EcoString,
}

/// Type path
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TypePath {
    Local {
        location: Address,
        name: EcoString,
    },
    Module {
        location: Address,
        module: EcoString,
        name: EcoString,
    },
    Function {
        location: Address,
        params: Vec<TypePath>,
        ret: Box<TypePath>,
    },
}

/// Type path implementation
impl TypePath {
    pub fn get_location(&self) -> Address {
        match self {
            TypePath::Local { location, .. } => location.clone(),
            TypePath::Module { location, .. } => location.clone(),
            TypePath::Function { location, .. } => location.clone(),
        }
    }
}

/// Parameter
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Parameter {
    pub location: Address,
    pub name: EcoString,
    pub typ: TypePath,
}

/// Enum constructor
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnumConstructor {
    pub location: Address,
    pub name: EcoString,
    pub params: Vec<Parameter>,
}

/// Publicity
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Publicity {
    Public,
    Private,
    None,
}

/// Pattern
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Pattern {
    // Unwrap enum pattern
    // `Pot.Full { flower, .. }`
    Unwrap {
        en: Expression,
        fields: Vec<(Address, EcoString)>,
    },
    // Enum variants
    Variant(Expression),
    // `123456`
    Int(EcoString),
    // `1.34`
    Float(EcoString),
    // Bool value `true` / `false
    Bool(EcoString),
    // "Hello, world!"
    String(EcoString),
    // Bind pattern
    BindTo(EcoString),
    // Default pattern
    Default,
}

/// Case
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Case {
    pub address: Address,
    pub pattern: Pattern,
    pub body: Block,
}

/// Use kind
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UseKind {
    AsName(EcoString),
    ForNames(Vec<EcoString>),
}

/// Else branch
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ElseBranch {
    Elif {
        location: Address,
        logical: Expression,
        body: Block,
    },
    Else {
        location: Address,
        body: Block,
    },
}

/// Expression
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expression {
    /// Represents `1x01231, 2101, 31...`
    /// int value
    Int { location: Address, value: EcoString },
    /// Represents `1.3, 4.5, 6.78..`
    /// int value
    Float { location: Address, value: EcoString },
    /// Represents "string" value
    String { location: Address, value: EcoString },
    /// Represents `true` or `false`
    /// value
    Bool { location: Address, value: EcoString },
    /// Represents binary expression
    ///
    /// `a || b`
    /// `a + b`
    ///
    /// ...
    Bin {
        location: Address,
        left: Box<Expression>,
        right: Box<Expression>,
        op: EcoString,
    },
    /// Represents unary expression
    ///
    /// -expr
    /// !expr
    ///
    Unary {
        location: Address,
        value: Box<Expression>,
        op: EcoString,
    },
    /// Represents if expression
    ///
    /// if `cond` {
    ///     ...
    /// }
    /// elif `cond` {
    ///     ...
    /// }
    /// ...
    /// else {
    ///     ...
    /// }
    If {
        location: Address,
        logical: Box<Expression>,
        body: Block,
        else_branches: Vec<ElseBranch>,
    },
    /// Represents prefix variable
    ///
    /// `name.`
    PrefixVar { location: Address, name: EcoString },
    /// Represents suffix variable
    ///
    /// `.name`
    SuffixVar {
        location: Address,
        container: Box<Expression>,
        name: EcoString,
    },
    /// Represents call expression
    ///
    /// `var_expr`()
    Call {
        location: Address,
        what: Box<Expression>,
        args: Vec<Expression>,
    },
    /// Represents anonymous function
    ///
    /// fn(...) {
    ///     ...
    /// }
    ///
    Function {
        location: Address,
        params: Vec<Parameter>,
        body: Block,
        typ: Option<TypePath>,
    },
    /// Match expression
    ///
    /// match ... {
    ///     ... -> ...,
    ///     ... -> ...,
    ///     _ -> ...
    /// }
    ///
    Match {
        location: Address,
        value: Box<Expression>,
        cases: Vec<Case>,
    },
}

/// Implementation
impl Expression {
    pub fn location(&self) -> Address {
        match self {
            Expression::Int { location, .. } => location.clone(),
            Expression::Float { location, .. } => location.clone(),
            Expression::String { location, .. } => location.clone(),
            Expression::Bool { location, .. } => location.clone(),
            Expression::Bin { location, .. } => location.clone(),
            Expression::Unary { location, .. } => location.clone(),
            Expression::If { location, .. } => location.clone(),
            Expression::PrefixVar { location, .. } => location.clone(),
            Expression::SuffixVar { location, .. } => location.clone(),
            Expression::Call { location, .. } => location.clone(),
            Expression::Function { location, .. } => location.clone(),
            Expression::Match { location, .. } => location.clone(),
        }
    }
}

/// Statement
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Statement {
    /// Definition statement
    ///
    /// let `name` = `value`
    ///
    VarDef {
        location: Address,
        name: EcoString,
        value: Expression,
        typ: Option<TypePath>,
    },
    /// Assignment statement
    ///
    /// `name` = `value`
    ///
    VarAssign {
        location: Address,
        what: Expression,
        value: Expression,
    },
    /// Expression
    ///
    /// represents expression
    Expr(Expression),
    /// Represents while loop
    ///
    /// while `cond` {
    ///     ...
    /// }
    ///
    While {
        location: Address,
        logical: Expression,
        body: Block,
    },
    /// Represents loop break
    ///
    /// `break`
    Break { location: Address },
    /// Represents loop continue
    ///
    /// `continue`
    Continue { location: Address },
    /// Represents semi colon statement
    Semi { stmt: Box<Statement> },
}

/// Block
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Block {
    pub location: Address,
    pub body: Vec<Statement>,
}

/// Represents use declaration
///
///  ... `as ...`, `for ..., ..., n`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Dependency {
    pub location: Address,
    pub path: DependencyPath,
    pub kind: UseKind,
}

/// Field
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Field {
    location: Address,
    publicity: Publicity,
    name: EcoString,
    value: Expression,
    typ: Option<TypePath>,
}

/// Method
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Method {
    location: Address,
    publicity: Publicity,
    name: EcoString,
    params: Vec<Parameter>,
    body: Block,
    typ: Option<TypePath>,
}

/// Declaration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Declaration {
    /// Represents type declaration
    ///
    /// `publicity` type ... {
    ///     `publicity` let ... = ...
    ///     `publicity` fn ... {
    ///         ...
    ///     }
    /// }
    ///
    TypeDeclaration {
        location: Address,
        name: EcoString,
        publicity: Publicity,
        constructor: Vec<Parameter>,
        declarations: Vec<Declaration>,
    },
    /// Represents enum declaration
    ///
    /// `publicity` enum ... {
    ///     ...(),
    ///     ...,
    ///     n
    /// }
    ///
    EnumDeclaration {
        location: Address,
        name: EcoString,
        publicity: Publicity,
        variants: Vec<EnumConstructor>,
    },
    /// Represents extern function declaration
    ///
    /// `publicity` extern fn(..., ..., n): typ = '""' / '``'
    ///
    ExternFn {
        location: Address,
        name: EcoString,
        publicity: Publicity,
        params: Vec<Parameter>,
        typ: Option<TypePath>,
        body: EcoString,
    },
    /// Definition statement
    ///
    /// let `name` = `value`
    ///
    VarDef {
        location: Address,
        publicity: Publicity,
        name: EcoString,
        value: Expression,
        typ: Option<TypePath>,
    },
    /// Function definition
    Function {
        location: Address,
        publicity: Publicity,
        name: EcoString,
        params: Vec<Parameter>,
        body: Block,
        typ: Option<TypePath>,
    },
}

/// Ast tree
#[derive(Debug)]
pub struct Module {
    pub source: NamedSource<Arc<String>>,
    pub dependencies: Vec<Dependency>,
    pub declarations: Vec<Declaration>,
}
