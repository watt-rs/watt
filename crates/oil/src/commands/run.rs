/// Imports
use camino::Utf8PathBuf;
use oil_pm::{compile::compile, runtime::JsRuntime};

/// Runs code
pub fn run(path: Utf8PathBuf, name: String, runtime: JsRuntime) {
    // Compiling code
    compile(path, name, runtime);
}
