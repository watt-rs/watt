/// Imports
use crate::{
    config::{self, PackageType}, url::path_to_pkg_name,
};
use camino::Utf8PathBuf;
use watt_compile::io;

/// Generates project
pub fn gen_project(path: Utf8PathBuf, ty: PackageType) {
    // Generating project
    let name = path_to_pkg_name(&path);
    match ty {
        PackageType::Lib => {
            // Generating config
            config::generate(&path, &name, ty, None);

            // Generating src path
            let src = path.join(name);
            io::mkdir(&src);

            // Generating main.wt
            let lib_wt = src.join("main.wt");
            io::write(
                &lib_wt,
                    r#"// `main.wt` is the main file of library project.

"#,
            );
        }
        PackageType::App => {
            // Generating config
            let main = format!("{}/{}", name, "main");
            config::generate(&path, &name, ty, Some(main));

            // Generating src path
            let src = path.join(name);
            io::mkdir(&src);

            // Generating main.wt
            let main = src.join("main.wt");
            io::write(
                &main,
                    r#"// `main.wt` is the starting point for your application.

fn main() {
    // Your code goes here.
}
"#,
            );
        }
    }
}
