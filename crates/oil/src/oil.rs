/// Imports
use camino::Utf8PathBuf;
use oil_compile::project::ProjectCompiler;

/// Runs code
#[allow(unused_variables)]
pub fn run() {
    let mut project_compiler = ProjectCompiler::new(
        vec![
            Utf8PathBuf::from("/home/vyacheslav/oil/std"),
            Utf8PathBuf::from("/home/vyacheslav/oil/tmp/test"),
        ],
        Utf8PathBuf::from("/home/vyacheslav/oil/tmp/target"),
    );
    project_compiler.compile();
}
