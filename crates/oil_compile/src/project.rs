/// Imports
use crate::{
    analyze::{rc_ptr::RcPtr, typ::Module},
    package::PackageCompiler,
};
use camino::Utf8PathBuf;
use ecow::EcoString;
use log::{info, trace};
use std::collections::HashMap;

/// Project compiler
pub struct ProjectCompiler {
    /// Sources
    pub packages: Vec<Utf8PathBuf>,
    /// Outcome
    pub outcome: Utf8PathBuf,
    /// Completed modules map
    pub modules: HashMap<EcoString, RcPtr<Module>>,
}

/// Project compiler implementation
impl ProjectCompiler {
    /// Creates new project compiler
    pub fn new(packages: Vec<Utf8PathBuf>, outcome: Utf8PathBuf) -> Self {
        Self {
            packages,
            outcome,
            modules: HashMap::new(),
        }
    }

    /// Compiles project
    pub fn compile(&mut self) {
        // Initializing logging
        pretty_env_logger::init();
        // Compiling
        trace!("Compiling project...");
        for package_path in &self.packages {
            PackageCompiler::new(
                package_path.clone(),
                self.outcome.clone(),
                &mut self.modules,
            )
            .compile();
        }
        info!("Done");
    }
}
