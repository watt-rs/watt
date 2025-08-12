/// Imports
use camino::Utf8PathBuf;
use oil_compile::{
    package::{PackageCompiler, PackageConfig},
    project::ProjectCompiler,
};

/// Runs code
#[allow(unused_variables)]
pub fn run(path: Utf8PathBuf, lex_debug: bool, parse_debug: bool) {
    let mut project_compiler = ProjectCompiler::new();
    let mut compiler = PackageCompiler::new(
        &mut project_compiler,
        PackageConfig {
            main: "main.oil".into(),
            version: "0.0.1".into(),
        },
        Utf8PathBuf::from("/home/vyacheslav/oil/tmp/"),
        Utf8PathBuf::from("/home/vyacheslav/oil/tmp/.out"),
    );
    compiler.compile();
}
