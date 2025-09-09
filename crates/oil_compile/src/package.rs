/// Imports
use crate::{
    analyze::{analyze::ModuleAnalyzer, typ::Module},
    errors::CompileError,
    io::io::{self, OilFile},
    project::ProjectCompiler,
};
use camino::Utf8PathBuf;
use ecow::EcoString;
use log::info;
use miette::NamedSource;
use oil_common::bail;
use oil_ir::{ir::IrModule, lowering};
use oil_lex::lexer::Lexer;
use oil_parse::parser::Parser;
use petgraph::{prelude::DiGraphMap, Direction};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

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
    fn load_module(&mut self, module_name: &EcoString, file: &OilFile) -> IrModule {
        // Reading code
        let code = file.read();
        let code_chars: Vec<char> = code.chars().collect();
        // Creating named source for miette
        let named_source = NamedSource::<Arc<String>>::new(module_name, Arc::new(code));
        // Lexing
        let lexer = Lexer::new(&code_chars, &named_source);
        let tokens = lexer.lex();
        // Parsing
        let mut parser = Parser::new(tokens, &named_source);
        let tree = parser.parse();
        // Untyped ir
        let untyped_ir = lowering::tree_to_ir(named_source, tree);
        return untyped_ir;
    }

    /// Collects all .oil files of package
    fn collect_sources(&self) -> Vec<OilFile> {
        io::collect_sources(&self.path)
    }

    /// Finds cycle in a graph
    fn find_cycle<'dep>(
        origin: &'dep EcoString,
        parent: &'dep EcoString,
        graph: &petgraph::prelude::DiGraphMap<&'dep EcoString, ()>,
        path: &mut Vec<&'dep EcoString>,
        done: &mut HashSet<&'dep EcoString>,
    ) -> bool {
        done.insert(parent);
        for node in graph.neighbors_directed(parent, Direction::Outgoing) {
            if node == origin {
                path.push(node);
                return true;
            }
            if done.contains(&node) {
                continue;
            }
            if Self::find_cycle(origin, node, graph, path, done) {
                path.push(node);
                return true;
            }
        }
        false
    }

    /// Toposorts dependencies graph
    fn toposort<'s>(&self, deps: HashMap<&'s EcoString, Vec<&'s EcoString>>) -> Vec<&'s EcoString> {
        // Creating graph for toposorting
        let mut deps_graph: DiGraphMap<&EcoString, ()> =
            petgraph::prelude::DiGraphMap::with_capacity(deps.len(), deps.len() * 5);

        // Adding nodes
        for key in deps.keys() {
            deps_graph.add_node(key);
        }
        for values in deps.values() {
            for v in values {
                deps_graph.add_node(v);
            }
        }

        // Adding edges
        for (key, values) in &deps {
            for dep in values {
                deps_graph.add_edge(key, dep, ());
            }
        }

        // Performing toposort
        match petgraph::algo::toposort(&deps_graph, None) {
            Ok(order) => order.into_iter().rev().collect(),
            Err(e) => {
                // Origin node
                let origin = e.node_id();
                // Cycle path
                let mut path = Vec::new();
                // Finding cycle
                if Self::find_cycle(origin, origin, &deps_graph, &mut path, &mut HashSet::new()) {
                    path.reverse();
                    bail!(CompileError::FoundImportCycle {
                        a: match path.get(0) {
                            Some(some) => (*some).clone(),
                            None =>
                                bail!(CompileError::CyclePathHasWrongLength { len: path.len() }),
                        },
                        b: match path.get(1) {
                            Some(some) => (*some).clone(),
                            None =>
                                bail!(CompileError::CyclePathHasWrongLength { len: path.len() }),
                        }
                    })
                } else {
                    bail!(CompileError::FailedToFindImportCycle)
                }
            }
        }
    }

    /// Compiles package
    pub fn compile(&mut self) {
        // Initializing logging
        pretty_env_logger::init();
        info!("compiling package: {}", self.path);

        // Collecting sources
        let mut modules = HashMap::new();
        for source in self.collect_sources() {
            let module_name = io::module_name(&self.path, &source);
            let module = self.load_module(&module_name, &source);
            modules.insert(module_name.clone(), module);
            info!("loaded module {:?} with name {:?}", source, module_name);
        }

        // Building dependencies tree
        info!("building dependencies tree...");
        let mut dep_tree: HashMap<&EcoString, Vec<&EcoString>> = HashMap::new();
        modules.iter().for_each(|(n, m)| {
            dep_tree.insert(n, m.dependencies.iter().map(|d| &d.path).collect());
        });
        info!("found dependencies {:#?}", dep_tree);

        // Performing toposort
        let sorted = self.toposort(dep_tree);
        let sorted_modules = sorted
            .iter()
            .map(|m| match modules.get(*m) {
                Some(module) => (*m, module),
                None => bail!(CompileError::NoModuleFound { name: (*m).clone() }),
            })
            .collect::<HashMap<&EcoString, &IrModule>>();
        info!("performed toposort {:#?}", sorted_modules);

        // Performing analyze
        info!("analyzing modules...");
        let mut modules: HashMap<EcoString, Module> = HashMap::new();
        for (name, module) in sorted_modules {
            let mut analyzer = ModuleAnalyzer::new(module, name, &mut modules);
            let module = analyzer.analyze();
            modules.insert(name.clone(), module);
        }

        // Performing codegen
        info!("performing codegen...")
    }
}
