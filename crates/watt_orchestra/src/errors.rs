/// IMports
use miette::Diagnostic;
use thiserror::Error;

/// Orchestra error
#[derive(Debug, Error, Diagnostic)]
pub enum OrchestraError {
    #[error("found an imports cycle: `{a}` <-> `{b}`.")]
    #[diagnostic(code(compile::found_imports_cycle))]
    FoundImportsCycle { a: String, b: String },
}
