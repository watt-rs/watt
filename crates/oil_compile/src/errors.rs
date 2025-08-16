/// Imports
use ecow::EcoString;
use miette::Diagnostic;
use thiserror::Error;

/// Compile error
#[derive(Debug, Error, Diagnostic)]
pub enum CompileError {
    #[error("found an import cycle {a} <> {b}.")]
    #[diagnostic(code(compile::found_import_cycle))]
    FoundImportCycle { a: EcoString, b: EcoString },
    #[error("import cycle path has wrong length {len}.")]
    #[diagnostic(
        code(compile::cycle_path_has_wrong_length),
        help("please, file an issue on github."),
        url("https://github.com/oillanguage/oil")
    )]
    CyclePathHasWrongLength { len: usize },
    #[error("import cycle is exists, but cannot be found.")]
    #[diagnostic(
        code(compile::failed_to_find_import_cycle),
        help("please, file an issue on github."),
        url("https://github.com/oillanguage/oil")
    )]
    FailedToFindImportCycle,
    #[error("no main module with name {main} found.")]
    #[diagnostic(code(compile::no_main_module_found), help("check module existence."))]
    NoMainModuleFound { main: EcoString },
    #[error("no module with name {name} found.")]
    #[diagnostic(code(compile::no_module_found), help("check module existence."))]
    NoModuleFound { name: EcoString },
}
