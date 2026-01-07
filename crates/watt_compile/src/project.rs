/// Imports
use crate::{
    io,
    package::{CompletedPackage, PackageCompiler},
};
use camino::Utf8PathBuf;
use ecow::EcoString;
use std::collections::HashMap;
use tracing::info;
use watt_common::{package::DraftPackage, rc_ptr::RcPtr};
use watt_typeck::{
    cx::root::RootCx,
    typ::{cx::TyCx, typ::Module},
};

/// Project compiler
pub struct ProjectCompiler<'out> {
    /// Sources
    pub packages: Vec<DraftPackage>,
    /// Outcome
    pub outcome: &'out Utf8PathBuf,
    /// Completed modules map
    pub modules: HashMap<EcoString, RcPtr<Module>>,
}

/// Project compiler implementation
impl<'out> ProjectCompiler<'out> {
    /// Creates new project compiler
    pub fn new(packages: Vec<DraftPackage>, outcome: &'out Utf8PathBuf) -> Self {
        Self {
            packages,
            outcome,
            modules: HashMap::new(),
        }
    }

    /// Writes `prelude.js`
    pub fn write_prelude(&mut self) {
        // Preludes path
        let mut preludes_path = self.outcome.clone();
        preludes_path.push("prelude.js");
        // Writing
        io::write(
            preludes_path,
            watt_gen::gen_prelude().to_file_string().unwrap(),
        );
    }

    /// Compiles project
    pub fn compile(&mut self) -> Vec<CompletedPackage> {
        // Compiling
        info!("Compiling project...");
        // Context
        let mut root_cx = RootCx::default();
        // Types context
        let mut tcx = TyCx::default();
        // Compiling packages
        let mut completed_packages = Vec::new();
        for package in &self.packages {
            completed_packages.push(
                PackageCompiler::new(
                    package.clone(),
                    self.outcome.clone(),
                    &mut root_cx,
                    &mut tcx,
                )
                .compile(),
            );
        }
        // Writing prelude
        self.write_prelude();
        // Done, returning result
        info!("Done");
        completed_packages
    }
}
