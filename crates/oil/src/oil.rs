/// Imports
use camino::Utf8PathBuf;
use oil_compile::project::ProjectCompiler;

use crate::commands::run;

/// Runs code
#[allow(unused_variables)]
pub fn run() {
    /*
    let mut project_compiler = ProjectCompiler::new(
        vec![
            Utf8PathBuf::from("/home/vyacheslav/oil/tmp/.cache/std/std"),
            Utf8PathBuf::from("/home/vyacheslav/oil/tmp/test"),
        ],
        Utf8PathBuf::from("/home/vyacheslav/oil/tmp/target"),
    );
    project_compiler.compile();
    */
    run::run(
        Utf8PathBuf::from("/home/vyacheslav/oil/tmp/"),
        String::from("test"),
    );
}
