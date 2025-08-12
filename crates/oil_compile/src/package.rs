/// Imports
use crate::{
    io::io::{self, OilFile},
    project::ProjectCompiler,
};
use camino::Utf8PathBuf;
use ecow::EcoString;
use log::{info, log};
use miette::NamedSource;
use oil_analyze::untyped_ir::{self, untyped_ir::{Dependency, UntypedModule}};
use oil_lex::lexer::Lexer;
use oil_parse::parser::Parser;
use std::{collections::HashMap, fs};

/// Package config
pub struct PackageConfig {
    /// Main file
    pub main: EcoString,
    /// Package version
    pub version: EcoString,
}

/// Package compiler
pub struct PackageCompiler<'compile> {
    /// Project compiler ref
    project: &'compile mut ProjectCompiler,
    /// Config of package
    config: PackageConfig,
    /// Path to package
    path: Utf8PathBuf,
    /// Compilation outcome path
    outcome: Utf8PathBuf,
}

/// Package compiler implementation
impl<'project_compiler> PackageCompiler<'project_compiler> {
    /// Creates new package compiler
    pub fn new(
        project: &'project_compiler mut ProjectCompiler,
        config: PackageConfig,
        path: Utf8PathBuf,
        outcome: Utf8PathBuf,
    ) -> Self {
        Self {
            project,
            config,
            path,
            outcome,
        }
    }

    /// Loads module
    fn load_module(&mut self, module_name: &EcoString, file: &OilFile) -> UntypedModule {
        // Reading code
        let code = file.read();
        let code_chars: Vec<char> = code.chars().collect();
        // Creating named source for miette
        let named_source = NamedSource::<String>::new(module_name, code);
        // Lexing
        let lexer = Lexer::new(&code_chars, &named_source);
        let tokens = lexer.lex();
        // Parsing
        let mut parser = Parser::new(tokens, &named_source);
        let tree = parser.parse();
        // Untyped ir
        let untyped_ir = untyped_ir::lowering::tree_to_ir(&named_source, tree);
        return untyped_ir;
    }

    /// Collects all .oil files of package
    fn collect_sources(&self) -> Vec<OilFile> {
        io::collect_sources(&self.path)
    }

    /// Compiles package
    pub fn compile(&mut self) {
        // Initializing logging
        pretty_env_logger::init();
        info!("compiling package: {}", self.path);

        // Collecting sources
        let modules = HashMap::new();
        for source in self.collect_sources() {
            let module_name = io::module_name(&self.path, &source);
            let module = self.load_module(&module_name, &source);
            modules.insert(module_name.clone(), module);
            info!("loaded module {:?} with name {:?}", source, module_name);
        }

        // Building dependencies tree
        info!("building dependencies tree...");
        let dependencies: HashMap<EcoString, Vec<Dependency>> = HashMap::new();
        modules.iter().for_each(|(n,m)| dependencies.insert(n, m.dependencies));
        info!("found dependencies {}", deps);
    }
}
