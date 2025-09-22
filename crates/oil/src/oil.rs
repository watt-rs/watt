use crate::commands::run;
/// Imports
use camino::Utf8PathBuf;

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
        Utf8PathBuf::from("/home/vyacheslav/oil/test/"),
        String::from("test"),
    );
}
