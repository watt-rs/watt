/// Imports
use ecow::EcoString;

/// Prelude type
#[derive(Debug, Clone)]
pub enum PreludeType {
    Int,
    Float,
    Bool,
    String,
}

/// Type
#[derive(Debug, Clone)]
pub enum Typ {
    Prelude(PreludeType),
    Defined(EcoString),
}

/// Module analyzer
pub struct ModuleAnalyzer {}
