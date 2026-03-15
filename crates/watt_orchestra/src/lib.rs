/// Modules
mod errors;
mod io;

/// Imports
use camino::Utf8PathBuf;
use miette::NamedSource;
use petgraph::{Direction, prelude::DiGraphMap};
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tracing::info;
use watt_ast::item;
use watt_lex::Lexer;
use watt_macros::{bail, bug};
use watt_parse::Parser;

use crate::errors::OrchestraError;

/// Orchestrator configuration
pub struct OrchestratorConfig {
    /// Compilation income path
    income: Utf8PathBuf,

    /// Compilation outcome path
    outcome: Utf8PathBuf,
}

/// Orchestrator config implementation
impl OrchestratorConfig {
    /// Creates new orchestrator config
    pub fn new(income: Utf8PathBuf, outcome: Utf8PathBuf) -> Self {
        Self { income, outcome }
    }
}

/// Represents orchestrator of the compilation or analysis
pub struct Orchestrator {
    config: OrchestratorConfig,
}

/// Orchestrator implementation
impl Orchestrator {
    /// Creates new orchestrator
    pub fn new(config: OrchestratorConfig) -> Self {
        Self { config }
    }

    /// Loads module and parses it
    fn load_module(&self, path: Utf8PathBuf) -> (String, item::Module) {
        // Reading module info
        info!("loading module `{path}`");
        let name = io::module_name(&self.config.income, &path);
        let code = io::read(&path);
        let source = Arc::new(NamedSource::new(name.clone(), code.clone()));

        // Parsing module
        let lexer = Lexer::new(source.clone(), &code);
        let mut parser = Parser::new(source, lexer);
        let ast = parser.parse();

        // Done
        info!("loaded module `{path}` with name `{name}`");
        (name, ast)
    }

    /// Loads modules from directory
    fn load_modules(&mut self) -> HashMap<String, item::Module> {
        info!("loading modules from `{}`", self.config.income);

        io::collect_sources(&self.config.income)
            .into_par_iter()
            .map(|path| self.load_module(path))
            .collect()
    }

    // Builds dependencies tree
    fn build_deptree<'lm>(
        &self,
        loaded_modules: &'lm HashMap<String, item::Module>,
    ) -> HashMap<&'lm String, Vec<&'lm String>> {
        let mut dep_tree: HashMap<&String, Vec<&String>> = HashMap::new();
        loaded_modules.iter().for_each(|(n, m)| {
            dep_tree.insert(
                n,
                m.imports
                    .iter()
                    .filter(|d| loaded_modules.contains_key(&d.path.module))
                    .map(|d| &d.path.module)
                    .collect(),
            );
        });

        dep_tree
    }

    /// Finds cycle in a graph
    fn find_graph_cycle<'dep>(
        origin: &'dep String,
        parent: &'dep String,
        graph: &petgraph::prelude::DiGraphMap<&'dep String, ()>,
        path: &mut Vec<&'dep String>,
        done: &mut HashSet<&'dep String>,
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
            if Self::find_graph_cycle(origin, node, graph, path, done) {
                path.push(node);
                return true;
            }
        }
        false
    }

    /// Performs toposort on imports/dependencies graph
    fn perform_toposort<'s>(&self, deps: HashMap<&'s String, Vec<&'s String>>) -> Vec<&'s String> {
        // Creating graph for toposorting
        let mut deps_graph: DiGraphMap<&String, ()> =
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
                if Self::find_graph_cycle(
                    origin,
                    origin,
                    &deps_graph,
                    &mut path,
                    &mut HashSet::new(),
                ) {
                    path.reverse();
                    bail!(OrchestraError::FoundImportsCycle {
                        a: match path.first() {
                            Some(some) => (*some).clone(),
                            None => bug!(format!("cycle path has wrong length: {}", path.len())),
                        },
                        b: match path.get(1) {
                            Some(some) => (*some).clone(),
                            None => bug!(format!("cycle path has wrong length: {}", path.len())),
                        }
                    })
                } else {
                    bug!("failed to find imports cycle")
                }
            }
        }
    }

    /// Performs compilation
    pub fn perform_compilation(&mut self) {
        // Loading modules
        info!(
            "starting compilation of `{}` with outcome `{}`",
            self.config.income, self.config.outcome
        );
        let loaded_modules = self.load_modules();

        // Building dependencies tree
        info!("building dependencies tree...");
        let dep_tree = self.build_deptree(&loaded_modules);

        // Performing toposort
        info!("performing toposort...");
        let sorted = self.perform_toposort(dep_tree);
        info!("performed toposort: {sorted:#?}");
    }
}
