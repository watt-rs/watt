/// Imports
use ecow::EcoString;
use miette::NamedSource;
use std::sync::Arc;
use watt_common::address::Address;

/// Dependency path
///
/// # Example
/// `this/is/some/module`
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DependencyPath {
    pub address: Address,
    pub module: EcoString,
}

/// Represents type path (type annotation)
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TypePath {
    /// Represents path to local user-defined
    /// or prelude type.
    ///
    /// # Example
    /// ```watt
    /// let a: int = 5;
    /// ```
    ///
    Local {
        location: Address,
        name: EcoString,
        generics: Vec<TypePath>,
    },
    /// Represents path to user-defined
    /// type used from module.
    ///
    /// # Example
    /// ```watt
    /// let a: module.Typ = 5;
    /// ```
    ///
    Module {
        location: Address,
        module: EcoString,
        name: EcoString,
        generics: Vec<TypePath>,
    },
    /// Represents function signature,
    /// used to determine function params and
    /// result type.
    ///
    /// # Example
    /// ```watt
    /// let sum: fn(int, int): int = fn(a: int, b: int) {
    ///     a + n
    /// };
    /// ```
    ///
    Function {
        location: Address,
        params: Vec<TypePath>,
        ret: Option<Box<TypePath>>,
    },
    /// Represents unit/none type.
    ///
    /// function returns
    /// that type by default
    ///
    Unit { location: Address },
}

/// Implementation
impl TypePath {
    pub fn location(&self) -> Address {
        match self {
            TypePath::Local { location, .. } => location.clone(),
            TypePath::Module { location, .. } => location.clone(),
            TypePath::Function { location, .. } => location.clone(),
            TypePath::Unit { location } => location.clone(),
        }
    }
}

/// Represents function or type parameter
/// as key value pair.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Parameter {
    /// Parameter name location
    ///
    /// ```watt
    /// fn some(a: int) {
    ///         ^ refs to this only
    /// }
    /// ```
    pub location: Address,
    /// Represents parameter name
    pub name: EcoString,
    /// Represents parameter type annotation
    ///
    /// ```watt
    /// fn some(a: int) {
    ///            ^^^ like this
    /// }
    /// ```
    pub typ: TypePath,
}

/// Enum constructor
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnumConstructor {
    /// Represents enum constructor location
    ///
    /// # Example
    /// ```watt
    /// enum Pot {
    ///     Full(flower: Flower),
    ///     ^^^^^^^^^^^^^^^^^^^^
    ///     this is s captured by span
    /// }
    /// ```
    pub location: Address,
    /// Represents variant name
    pub name: EcoString,
    /// Represents variant parameters / fields
    ///
    /// # Example
    /// ```watt
    /// enum Color {
    ///     Rgb(r: int, g: int, b: int),
    ///         ^^^^^^^^^^^^^^^^^^^^^^
    ///                this
    ///     Hex(code: string)
    /// }
    /// ```
    pub params: Vec<Parameter>,
}

/// Binary operator
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    /// +
    Add,
    /// -
    Sub,
    /// %
    Mul,
    /// /
    Div,
    /// %
    Mod,
    /// ==
    Eq,
    /// !=
    NotEq,
    /// >
    Gt,
    /// >=
    Ge,
    /// <
    Lt,
    /// <=
    Le,
    /// &&
    And,
    /// ||
    Or,
    /// ^
    Xor,
    /// &
    BitwiseAnd,
    /// |
    BitwiseOr,
    /// <>
    Concat,
}

/// Unary operator
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    /// -
    Neg,
    /// !
    Bang,
}

/// Publicity
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Publicity {
    /// Represents `pub` publicity
    ///
    /// # Example
    /// ```watt
    /// pub fn sum(a: int, b: int): int {
    /// ^^^
    /// this
    ///     a + b
    /// }
    /// ```
    ///
    Public,
    /// Represents default private publicity.
    Private,
}

/// Pattern
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Pattern {
    /// Represents enum fields unwrap pattern
    ///
    /// # Example
    /// ```watt
    /// let a = Option.Some(3);
    /// let result = match a {
    ///  Option.None -> -1,
    ///  Option.Some(value) -> value
    ///              ^^^^^
    ///               this
    /// };
    /// ```
    ///
    Unwrap {
        address: Address,
        en: Expression,
        fields: Vec<(Address, EcoString)>,
    },
    /// Represents just enum variant pattern
    ///
    /// # Example
    /// ```watt
    /// let a = Color.Rgb(3, 4, 5);
    /// let hex_code: dyn = match a {
    ///  Color.Hex(code) -> code,
    ///           ^^^^^
    ///         fields unwrap here
    ///  Color.Rgb -> ""
    ///        ^^^
    ///      no fields unwrap here.
    /// };
    /// ```
    ///
    /// Using variant pattern instead of Unwrap
    /// pattern with enum variant that contain fields
    /// is available. There won't be an error like in Rust.
    ///
    Variant(Address, Expression),
    /// Represents integer pattern, example: `123`
    Int(Address, EcoString),
    /// Represents float pattern, example: `1.34`
    Float(Address, EcoString),
    /// Represents bool pattern: `true` / `false
    Bool(Address, EcoString),
    /// Represents string pattern: "Hello, world!"
    String(Address, EcoString),
    /// Represents bind pattern
    ///
    /// # Example
    /// ```watt
    /// let a = Option.None();
    /// match a {
    ///  Option.Some -> ...
    ///  none -> ...
    ///  ^^^
    ///  passes any original value, that we matching
    ///  to the case body as variable with specified
    ///  named
    /// }
    /// ```
    BindTo(Address, EcoString),
    /// Represents wilcard pattern
    ///
    /// # Example
    /// ```watt
    /// let a = Option.None();
    /// match a {
    ///  Option.Some -> ...
    ///  _ -> ...
    ///  ^^^
    ///  it doesn't pass original value as variable with
    ///  specified name, instead of BindTo pattern. Wildcard pattern
    ///  just ignores that value.
    ///  We can describe this pattern like: Otherwise, Else, AnyOther
    /// };
    /// ```
    Wildcard,
    // Two patterns in one
    ///
    /// # Example 1
    /// ```watt
    /// enum Animal {
    ///    Bear,
    ///    Cat,
    ///    Dog,
    ///    Rabbit
    /// }
    ///
    /// fn is_pet(animal: Animal): bool {
    ///  match animal {
    ///   Animal.Cat | Animal.Dog -> true
    ///   ^^^^^^^^^^^^^^^^^^^^^^^
    ///      two patterns or more separated by bar
    ///      that will be represented as Or(a, Or(b, c)) pattern
    ///   _ -> false
    ///  }
    /// }
    /// ```
    ///
    /// # Example 2 (Important)
    /// ```watt
    /// enum Animal {
    ///    Bear,
    ///    Cat(food: int),
    ///    Dog(food: int),
    ///    Rabbit
    /// }
    ///
    /// fn is_pet(animal: Animal): bool {
    ///  match animal {
    ///   Animal.Cat(food) | Animal.Dog(food) -> true
    ///   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    ///      two patterns or more can be any patterns,
    ///      but if first pattern is Unwrap, all others should
    ///      be Unwrap patterns too. Moreover, both patterns
    ///      should unwrap fields with same names and types.
    ///
    ///   _ -> false
    ///  }
    /// }
    /// ```
    ///
    Or(Box<Pattern>, Box<Pattern>),
}

/// Case
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Case {
    /// Case location
    ///
    /// ```watt
    /// match a {
    ///     Option.Some(value) -> {  < captures full case body
    ///     }                        < with provided case pattern.
    /// }
    /// ```
    ///
    pub address: Address,
    /// Pattern
    ///
    /// ```watt
    /// match a {
    ///     Option.Some(value) -> {
    ///     ^^^^^^^^^^^^^^^^^^
    ///   this part is a pattern
    ///     }
    /// }
    /// ```
    pub pattern: Pattern,
    /// Body of case
    pub body: Either<Block, Expression>,
}

/// Use kind
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UseKind {
    /// Represents import of module as given name
    ///
    /// # Example
    /// ```watt
    /// use some/module as name
    /// ```
    AsName(EcoString),
    /// Represents import of module contents separated by commad
    ///
    /// # Example 1
    /// ```watt
    /// use some/module for Typ
    /// ```
    ///
    /// # Example 2
    /// ```watt
    /// use some/module for Typ, Ty2
    /// ```
    ForNames(Vec<EcoString>),
}

/// Else branch
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ElseBranch {
    Elif {
        location: Address,
        logical: Expression,
        body: Either<Block, Expression>,
    },
    Else {
        location: Address,
        body: Either<Block, Expression>,
    },
}

/// Range
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Range {
    /// If range excludes last value
    ///
    /// # Example
    /// ```watt
    /// 0..30
    /// ```
    ExcludeLast {
        location: Address,
        from: Expression,
        to: Expression,
    },
    /// If range includes last value
    ///
    /// # Example
    /// ```watt
    /// 0..=30
    /// ```
    IncludeLast {
        location: Address,
        from: Expression,
        to: Expression,
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
    /// Represents todo
    ///
    /// `todo`
    /// `todo as "some description"`
    ///
    /// ...
    Todo {
        location: Address,
        text: Option<EcoString>,
    },
    /// Represents panic
    ///
    /// `panic`
    /// `panic as "some description"`
    ///
    /// ...
    Panic {
        location: Address,
        text: Option<EcoString>,
    },
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
        op: BinaryOp,
    },
    /// Represents cast expression
    ///
    /// `a as t`
    ///
    As {
        location: Address,
        value: Box<Expression>,
        typ: TypePath,
    },
    /// Represents unary expression
    ///
    /// `-expr`
    /// `!expr`
    ///
    Unary {
        location: Address,
        value: Box<Expression>,
        op: UnaryOp,
    },
    /// Represents if expression
    ///
    /// ```watt
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
    /// ```
    If {
        location: Address,
        logical: Box<Expression>,
        body: Either<Block, Box<Expression>>,
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
        body: Either<Block, Box<Expression>>,
        typ: Option<TypePath>,
    },
    /// Match expression
    ///
    /// ```watt
    /// match ... {
    ///     ... -> ...,
    ///     ... -> ...,
    ///     _ -> ...
    /// }
    /// ```
    ///
    Match {
        location: Address,
        value: Box<Expression>,
        cases: Vec<Case>,
    },
    /// Paren expression
    ///
    /// ```watt
    /// (...)
    /// ```
    Paren {
        location: Address,
        expr: Box<Expression>,
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
            Expression::Panic { location, .. } => location.clone(),
            Expression::Todo { location, .. } => location.clone(),
            Expression::Bin { location, .. } => location.clone(),
            Expression::As { location, .. } => location.clone(),
            Expression::Unary { location, .. } => location.clone(),
            Expression::If { location, .. } => location.clone(),
            Expression::PrefixVar { location, .. } => location.clone(),
            Expression::SuffixVar { location, .. } => location.clone(),
            Expression::Call { location, .. } => location.clone(),
            Expression::Function { location, .. } => location.clone(),
            Expression::Match { location, .. } => location.clone(),
            Expression::Paren { location, .. } => location.clone(),
        }
    }
}

/// Either type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Either<A, B> {
    Left(A),
    Right(B),
}

/// Statement
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Statement {
    /// Definition statement
    ///
    /// ```watt
    /// let `name` = `value`
    /// ```
    ///
    VarDef {
        location: Address,
        name: EcoString,
        value: Expression,
        typ: Option<TypePath>,
    },
    /// Assignment statement
    ///
    /// ```watt
    /// `name` = `value`
    /// ```
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
    /// Represents loop
    ///
    /// ```watt
    /// loop `cond` {
    ///     ...
    /// }
    /// ```
    ///
    Loop {
        location: Address,
        logical: Expression,
        body: Either<Block, Expression>,
    },
    /// Represents `for` loop
    ///
    /// ```watt
    /// for `name` in `range` {
    ///     ...
    /// }
    /// ```
    ///
    For {
        location: Address,
        name: EcoString,
        range: Box<Range>,
        body: Either<Block, Expression>,
    },
    /// Represents semi colon expression
    Semi(Expression),
}

/// Implementation
impl Statement {
    pub fn location(&self) -> Address {
        match self {
            Statement::VarDef { location, .. } => location.clone(),
            Statement::VarAssign { location, .. } => location.clone(),
            Statement::Expr(expression) => expression.location(),
            Statement::Loop { location, .. } => location.clone(),
            Statement::For { location, .. } => location.clone(),
            Statement::Semi(expression) => expression.location(),
        }
    }
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
    pub location: Address,
    pub name: EcoString,
    pub typ: TypePath,
}

/// Type declaration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeDeclaration {
    /// Represents struct declaration
    ///
    /// `publicity` type ... {
    ///     ...,
    ///     ...,
    ///     n
    /// }
    ///
    Struct {
        location: Address,
        name: EcoString,
        publicity: Publicity,
        generics: Vec<EcoString>,
        fields: Vec<Field>,
    },
    /// Represents enum declaration
    ///
    /// `publicity` enum ... {
    ///     ...,
    ///     ...,
    ///     n
    /// }
    ///
    Enum {
        location: Address,
        name: EcoString,
        publicity: Publicity,
        generics: Vec<EcoString>,
        variants: Vec<EnumConstructor>,
    },
}

/// Function declaration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FnDeclaration {
    /// Function definition
    Function {
        location: Address,
        publicity: Publicity,
        name: EcoString,
        generics: Vec<EcoString>,
        params: Vec<Parameter>,
        body: Either<Block, Expression>,
        typ: Option<TypePath>,
    },
    /// Represents extern function declaration
    ///
    /// `publicity` extern fn(..., ..., n): typ = '""' / '``'
    ///
    ExternFunction {
        location: Address,
        name: EcoString,
        publicity: Publicity,
        generics: Vec<EcoString>,
        params: Vec<Parameter>,
        typ: Option<TypePath>,
        body: EcoString,
    },
}

/// Constant declaration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConstDeclaration {
    pub location: Address,
    pub publicity: Publicity,
    pub name: EcoString,
    pub value: Expression,
    pub typ: TypePath,
}

/// Declaration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Declaration {
    /// Type declaration
    Type(TypeDeclaration),
    /// Function declaration
    Fn(FnDeclaration),
    /// Constant declaration
    Const(ConstDeclaration),
}

/// Ast tree
#[derive(Debug)]
pub struct Module {
    pub source: Arc<NamedSource<String>>,
    pub dependencies: Vec<Dependency>,
    pub declarations: Vec<Declaration>,
}
