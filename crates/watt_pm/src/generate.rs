/// Imports
use crate::{
    compile::path_to_pkg_name,
    config::{self, PackageType},
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
            config::generate(path.clone(), &name, ty, None);
            // Generating src path
            let src = path.join(name);
            io::mkdir(&src);
        }
        PackageType::App => {
            // Generating config
            let main = format!("{}/{}", name, "main");
            config::generate(path.clone(), &name, ty, Some(main));
            // Generating src path
            let src = path.join(name);
            io::mkdir(&src);
            // Generating main.watt
            let main = src.join("main.watt");
            io::write(main, String::from("// It's just a main file :)"));
        }
    }
}
