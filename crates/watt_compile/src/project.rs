/// Imports
use crate::{
    io,
    package::{CompiledPackage, PackageCompiler},
};
use camino::Utf8PathBuf;
use tracing::info;
use watt_common::package::DraftPackage;
use watt_typeck::{cx::root::RootCx, typ::cx::TyCx};

/// Built type
pub type Built = Vec<CompiledPackage>;

/// Project compiler
pub struct ProjectCompiler<'out> {
    /// Root context
    pub rcx: RootCx,
    /// Sources
    pub packages: Vec<DraftPackage>,
    /// Outcome
    pub outcome: &'out Utf8PathBuf,
}

/// Project compiler implementation
impl<'out> ProjectCompiler<'out> {
    /// Creates new project compiler
    pub fn new(packages: Vec<DraftPackage>, outcome: &'out Utf8PathBuf) -> Self {
        Self {
            rcx: RootCx::default(),
            packages,
            outcome,
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
            &watt_gen::gen_prelude().to_file_string().unwrap(),
        );
    }

    /// Compiles project
    pub fn compile(&mut self) -> Built {
        // Compiling
        info!("Compiling project...");
        // Types context
        let mut tcx = TyCx::default();
        // Compiling packages
        let mut compiled_packages = Vec::new();
        for package in &self.packages {
            compiled_packages.push(
                PackageCompiler::new(
                    package.clone(),
                    self.outcome.clone(),
                    &mut self.rcx,
                    &mut tcx,
                )
                .compile(),
            );
        }
        // Writing prelude
        self.write_prelude();
        // Done, returning result
        info!("Done");
        compiled_packages
    }

    /// Analyzes project
    pub fn analyze(&mut self) {
        info!("Analyzing project...");
        // Context
        let mut root_cx = RootCx::default();
        // Types context
        let mut tcx = TyCx::default();
        // Compiling packages
        for package in &self.packages {
            PackageCompiler::new(
                package.clone(),
                self.outcome.clone(),
                &mut root_cx,
                &mut tcx,
            )
            .analyze();
        }

        // Done
        info!("Done");
    }
}
