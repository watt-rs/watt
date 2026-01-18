/// Imports
use crate::{
    errors::CompileError,
    io::{self, WattFile},
};
use camino::{Utf8Path, Utf8PathBuf};
use ecow::EcoString;
use miette::NamedSource;
use petgraph::{Direction, prelude::DiGraphMap};
use std::{
    collections::{HashMap, HashSet},
    fs,
    sync::Arc,
};
use tracing::{error, info};
use watt_ast::ast::{self};
use watt_common::{bail, package::DraftPackage, rc_ptr::RcPtr};
use watt_gen::gen_module;
use watt_lex::lexer::Lexer;
use watt_lint::lint::LintCx;
use watt_parse::parser::Parser;
use watt_typeck::{
    cx::{module::ModuleCx, package::PackageCx, root::RootCx},
    typ::{cx::TyCx, typ::Module},
};

/// Completed module
pub struct CompletedModule {
    /// Name
    pub name: EcoString,
    /// Analyzed
    pub analyzed: RcPtr<Module>,
    /// Generated
    pub generated: Utf8PathBuf,
}

/// Completed package
pub struct CompletedPackage {
    /// Path to package
    pub path: Utf8PathBuf,
    /// Completed modules
    pub modules: Vec<CompletedModule>,
}

/// Package compiler
pub struct PackageCompiler<'cx> {
    /// Compilation outcome path
    outcome: Utf8PathBuf,
    /// Package typeck cx
    package: PackageCx<'cx>,
    /// Types context
    tcx: &'cx mut TyCx,
}

/// Package compiler implementation
impl<'cx> PackageCompiler<'cx> {
    /// Creates new package compiler
    pub fn new(
        draft: DraftPackage,
        outcome: Utf8PathBuf,
        root: &'cx mut RootCx,
        tcx: &'cx mut TyCx,
    ) -> Self {
        Self {
            outcome,
            package: PackageCx { draft, root },
            tcx,
        }
    }

    /// Loads module
    fn load_module(&self, module_name: &EcoString, file: &WattFile) -> ast::Module {
        // Reading code
        let code = file.read();
        let code_chars: Vec<char> = code.chars().collect();
        // Creating named source for miette
        let named_source = Arc::new(NamedSource::<String>::new(module_name, code));
        // Lexing
        let lexer = Lexer::new(&code_chars, &named_source);
        let tokens = lexer.lex();
        // Parsing
        let mut parser = Parser::new(tokens, &named_source);
        let ast = parser.parse();
        // Linting
        let linter = LintCx::new(&self.package.draft, &ast);
        linter.lint();
        // Done
        ast
    }

    /// Collects all .watt files of package
    fn collect_sources(&self) -> Vec<WattFile> {
        io::collect_sources(&self.package.draft.path)
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
                        a: match path.first() {
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

    fn load_modules(&self) -> HashMap<EcoString, ast::Module> {
        let mut loaded_modules = HashMap::new();
        for source in self.collect_sources() {
            let module_name = io::module_name(&self.package.draft.path, &source);
            let module = self.load_module(&module_name, &source);
            loaded_modules.insert(module_name.clone(), module);
            info!("Loaded module {source:?} with name {module_name:?}");
        }

        loaded_modules
    }

    fn build_deptree<'mo>(&self, loaded_modules: &'mo HashMap<EcoString, ast::Module>) -> HashMap<&'mo EcoString, Vec<&'mo EcoString>> {
        let mut dep_tree: HashMap<&EcoString, Vec<&EcoString>> = HashMap::new();
        loaded_modules.iter().for_each(|(n, m)| {
            dep_tree.insert(
                n,
                m.dependencies
                    .iter()
                    .filter(|d| loaded_modules.contains_key(&d.path.module))
                    .map(|d| &d.path.module)
                    .collect(),
            );
        });

        dep_tree
    }

    fn analyze_modules<'s>(&'s mut self, sorted: Vec<&EcoString>, loaded_modules: &'s HashMap<EcoString, ast::Module>) -> HashMap<EcoString, RcPtr<Module>> {
        let mut analyzed_modules = HashMap::new();

        for name in sorted.into_iter() {
            info!("Analyzing module {name}");
            let module = loaded_modules.get(name).unwrap();
            let mut analyzer = ModuleCx::new(module, name, self.tcx, &self.package);
            let analyzed_module = RcPtr::new(analyzer.analyze());
            self.package
                .root
                .modules
                .insert(name.clone(), analyzed_module.clone());
            analyzed_modules.insert(name.clone(), analyzed_module);
        }

        analyzed_modules
    }

    /// Compiles package
    /// returns analyzed modules
    pub fn compile(&mut self) -> CompletedPackage {
        info!("Compiling package: {}", self.package.draft.path);

        // Collecting sources
        let loaded_modules = self.load_modules();

        // Building dependencies tree
        info!("Building dependencies tree...");

        let dep_tree = self.build_deptree(&loaded_modules);
        info!("Found dependencies {dep_tree:#?}");

        // Performing toposort
        let sorted = self.toposort(dep_tree);
        info!("Performed toposort {sorted:#?}");

        // Performing analyze
        info!("Analyzing modules...");
        let analyzed_modules = self.analyze_modules(sorted, &loaded_modules);

        // Performing codegen
        info!("Performing codegen...");
        let mut generated_modules = HashMap::new();
        for module in analyzed_modules.iter() {
            info!("Performing codegen for {}", module.0);
            let generated = gen_module(module.0, loaded_modules.get(module.0).unwrap())
                .to_file_string()
                .unwrap();
            generated_modules.insert(module.0.clone(), generated);
        }

        // Writing outcome
        info!("Writing outcome...");
        let mut completed_modules = HashMap::new();
        for module in generated_modules {
            // Target path
            let mut target_path = self.outcome.clone();
            target_path.push(Utf8Path::new(&format!("{}.js", &module.0)));
            completed_modules.insert(module.0, target_path.clone());
            // Creating directory
            if let Some(path) = target_path.parent() {
                // Catching error
                if let Err(error) = fs::create_dir_all(path) {
                    error!("{error:?}");
                }
            }
            // Creating file
            io::write(target_path, &module.1);
        }

        // Returning analyzed modules
        CompletedPackage {
            path: self.package.draft.path.clone(),
            modules: analyzed_modules
                .into_iter()
                .map(|(name, module)| CompletedModule {
                    name: name.clone(),
                    analyzed: module,
                    generated: completed_modules.get(&name).unwrap().clone(),
                })
                .collect(),
        }
    }

    pub fn analyze(&mut self) {
        info!("Analyzing package: {}", self.package.draft.path);

        // Collecting sources
        let loaded_modules = self.load_modules();

        // Building dependencies tree
        info!("Building dependencies tree...");

        let dep_tree = self.build_deptree(&loaded_modules);
        info!("Found dependencies {dep_tree:#?}");

        // Performing toposort
        let sorted = self.toposort(dep_tree);
        info!("Performed toposort {sorted:#?}");

        // Performing analyze
        info!("Analyzing modules...");
        self.analyze_modules(sorted, &loaded_modules);
    }
}
