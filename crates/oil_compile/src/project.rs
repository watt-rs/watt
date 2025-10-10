/// Imports
use crate::{
    analyze::{rc_ptr::RcPtr, typ::Module},
    io,
    package::{CompletedPackage, PackageCompiler},
};
use camino::Utf8PathBuf;
use ecow::EcoString;
use log::{info, trace};
use std::collections::HashMap;

/// Project compiler
pub struct ProjectCompiler<'out> {
    /// Sources
    pub packages: Vec<Utf8PathBuf>,
    /// Outcome
    pub outcome: &'out Utf8PathBuf,
    /// Completed modules map
    pub modules: HashMap<EcoString, RcPtr<Module>>,
}

/// Project compiler implementation
impl<'out> ProjectCompiler<'out> {
    /// Creates new project compiler
    pub fn new(packages: Vec<Utf8PathBuf>, outcome: &'out Utf8PathBuf) -> Self {
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
            oil_gen::gen_prelude().to_file_string().unwrap(),
        );
    }

    /// Compiles project
    pub fn compile(&mut self) -> Vec<CompletedPackage> {
        // Compiling
        trace!("Compiling project...");
        // Compiling packages
        let mut completed_packages = Vec::new();
        for package_path in &self.packages {
            completed_packages.push(
                PackageCompiler::new(
                    package_path.clone(),
                    self.outcome.clone(),
                    &mut self.modules,
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
