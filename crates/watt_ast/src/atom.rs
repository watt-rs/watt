/// Imports
use watt_lex::token::Span;

/// Represents item publicity (visibility modifier)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Publicity {
    /// Public item (accessible outside the module)
    Pub,

    /// Private item (accessible only inside the module)
    Priv,
}

/// Binary operator
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BinOp {
    Add,    // `+`
    Sub,    // `-`
    Mul,    // `*`
    Div,    // `/`
    Mod,    // `%`
    Eq,     // `==`
    Ne,     // `!=`
    Gt,     // `>`
    Ge,     // `>=`
    Lt,     // `<`
    Le,     // `<=`
    And,    // `&&`
    Or,     // `||`
    Xor,    // `^`
    BitAnd, // `&`
    BitOr,  // `|`
    Concat, // `<>`
}

/// Assignment operation used in assignment expressions
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AssignOp {
    /// `+=`
    AddEq,

    /// `-=`
    SubEq,

    /// `*=`
    MulEq,

    /// `/=`
    DivEq,

    /// `%=`
    ModEq,

    /// `&=`
    AndEq,

    /// `|=`
    OrEq,

    /// `^=`
    XorEq,

    /// `=`
    Eq,
}

/// Unary operator
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UnOp {
    Neg,  // -
    Bang, // !
}

/// Represents literal
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Lit {
    /// Integer literal
    Int(String),

    /// Floating-point literal
    Float(String),

    /// String literal
    String(String),

    /// Boolean literal (`true` or `false`)
    Bool(String),
}

/// Represents a type hint (type annotation)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeHint {
    /// Type defined in the current module
    Local {
        /// Span of the type in the source code
        span: Span,

        /// Type name
        name: String,

        /// Generic arguments
        args: Vec<TypeHint>,
    },

    /// Type imported from another module
    Module {
        /// Span of the type in the source code
        span: Span,

        /// Module name where the type is defined
        module: String,

        /// Type name
        name: String,

        /// Generic arguments
        args: Vec<TypeHint>,
    },

    /// Function type
    /// Example: `(int, int) -> int`
    Function {
        /// Span of the type
        span: Span,

        /// Parameter types
        params: Vec<TypeHint>,

        /// Return type
        ret: Box<TypeHint>,
    },

    /// Unit type `()`
    Unit(Span),

    /// Type should be inferred by the compiler
    Infer,
}

/// Represents a function parameter
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Param {
    /// Span of the parameter in the source code
    pub span: Span,

    /// Parameter name
    pub name: String,

    /// Type hint of the parameter
    pub hint: TypeHint,
}
