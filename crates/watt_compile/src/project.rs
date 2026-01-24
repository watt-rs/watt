/// Imports
use crate::{
    io,
    package::{CompiledPackage, PackageCompiler},
};
use camino::Utf8PathBuf;
use tracing::info;
use watt_common::package::DraftPackage;
use watt_typeck::{cx::root::RootCx, typ::cx::TyCx};

/// Build represents final compilation output,
/// that contains context to access modules by its IDs
/// and vector of `CompiledPackage`, packages,
pub struct Built {
    pub rcx: RootCx,
    pub compiled: Vec<CompiledPackage>,
}

/// Implementation
impl Built {
    pub fn new(rcx: RootCx, compiled: Vec<CompiledPackage>) -> Built {
        Built { rcx, compiled }
    }
}

/// Project compiler
pub struct ProjectCompiler<'out> {
    /// Sources
    pub packages: Vec<DraftPackage>,
    /// Outcome
    pub outcome: &'out Utf8PathBuf,
}

/// Project compiler implementation
impl<'out> ProjectCompiler<'out> {
    /// Creates new project compiler
    pub fn new(packages: Vec<DraftPackage>, outcome: &'out Utf8PathBuf) -> Self {
        Self { packages, outcome }
    }

    /// Writes `prelude.js`
    pub fn write_prelude(&mut self) {
        // Preludes path
        let mut preludes_path = self.outcome.clone();
        preludes_path.push("prelude.js");
        // Writing
        io::write(
            preludes_path,
            &watt_gen::gen_prelude().to_file_string().unwrap(),
        );
    }

    /// Compiles project
    pub fn compile(&mut self) -> Built {
        // Compiling
        info!("Compiling project...");
        // Context
        let mut rcx = RootCx::default();
        // Types context
        let mut tcx = TyCx::default();
        // Compiling packages
        let mut compiled_packages = Vec::new();
        for package in &self.packages {
            compiled_packages.push(
                PackageCompiler::new(package.clone(), self.outcome.clone(), &mut rcx, &mut tcx)
                    .compile(),
            );
        }
        // Writing prelude
        self.write_prelude();
        // Done, returning result
        info!("Done");
        Built::new(rcx, compiled_packages)
    }

    /// Analyzes project
    pub fn analyze(&mut self) {
        info!("Analyzing project...");
        // Context
        let mut rcx = RootCx::default();
        // Types context
        let mut tcx = TyCx::default();
        // Compiling packages
        for package in &self.packages {
            PackageCompiler::new(package.clone(), self.outcome.clone(), &mut rcx, &mut tcx)
                .analyze();
        }
        // Done
        info!("Done");
    }
}
