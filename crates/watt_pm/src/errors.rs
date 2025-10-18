/// Imports
use crate::runtime::JsRuntime;
use camino::Utf8PathBuf;
use miette::Diagnostic;
use thiserror::Error;

/// Package error
#[derive(Debug, Error, Diagnostic)]
pub enum PackageError {
    #[error("failed to parse \"watt.toml\" at \"{path}\"")]
    #[diagnostic(code(pkg::failed_to_parse_config))]
    FailedToParseConfig { path: Utf8PathBuf },
    #[error("failed to find \"watt.toml\" at \"{path}\"")]
    #[diagnostic(code(pkg::failed_to_find_config))]
    FailedToFindConfig { path: Utf8PathBuf },
    #[error("failed to generate \"watt.toml\" at \"{path}\". file already exists.")]
    #[diagnostic(code(pkg::failed_to_gen_config))]
    FailedToGenConfig { path: Utf8PathBuf },
    #[error("failed to serialize config.")]
    #[diagnostic(
        code(pkg::failed_to_serialize_config),
        help("please, file an issue on github."),
        url("https://github.com/wattlanguage/watt")
    )]
    FailedToSerializeConfig { path: Utf8PathBuf },
    #[error("found an dependencies cycle \"{a}\" <> \"{b}\".")]
    #[diagnostic(code(pkg::found_import_cycle))]
    FoundDependenciesCycle { a: String, b: String },
    #[error("import cycle path has wrong length {len}.")]
    #[diagnostic(
        code(pkg::cycle_path_has_wrong_length),
        help("please, file an issue on github."),
        url("https://github.com/wattlanguage/watt")
    )]
    CyclePathHasWrongLength { len: usize },
    #[error("url \"{url}\" is invalid.")]
    #[diagnostic(code(pkg::invalid_url))]
    InvalidUrl { url: String },
    #[error("failed to clone repository from \"{url}\".")]
    #[diagnostic(code(pkg::failed_to_clone_repo))]
    FailedToCloneRepo { url: String },
    #[error("import cycle is exists, but cannot be found.")]
    #[diagnostic(
        code(pkg::failed_to_find_import_cycle),
        help("please, file an issue on github."),
        url("https://github.com/wattlanguage/watt")
    )]
    FailedToFindDependenciesCycle,
    #[error("dependency key \"{key}\" is not found in solved map.")]
    #[diagnostic(
        code(pkg::no_solved_key_found),
        help("please, file an issue on github."),
        url("https://github.com/wattlanguage/watt")
    )]
    NoSolvedKeyFound { key: String },
    #[error("failed to run project using {rt:?}. error: {error}")]
    #[diagnostic(code(pkg::failed_to_run_project))]
    FailedToRunProject { rt: JsRuntime, error: String },
    #[error("no main package with path {path} found.")]
    #[diagnostic(
        code(compile::no_main_package_found),
        help("please, file an issue on github."),
        url("https://github.com/wattlanguage/watt")
    )]
    NoMainPackageFound { path: Utf8PathBuf },
    #[error("no main module with name \"{module}\" found.")]
    #[diagnostic(code(pkg::no_main_module_found), help("check module existence."))]
    NoMainModuleFound { module: String },
    #[error("no main function found in module \"{module}\" marked as main.")]
    #[diagnostic(code(pkg::no_main_function_found), help("define a main function."))]
    NoMainFnFound { module: String },
    #[error("no main module specified in config {path}.")]
    #[diagnostic(
        code(pkg::no_main_module_specified),
        help("please, specify the module in config.")
    )]
    NoMainModuleFoundSpecified { path: Utf8PathBuf },
    #[error("failed to get project name from path {path}.")]
    #[diagnostic(code(pkg::failed_to_get_project_name_from_path))]
    FailedToGetProjectNameFromPath { path: Utf8PathBuf },
    #[error("could not use package \"{name}\" with package type \"app\" as dependency.")]
    #[diagnostic(code(pkg::use_of_app_package_as_dependency))]
    UseOfAppPackageAsDependency { name: String, path: Utf8PathBuf },
}
