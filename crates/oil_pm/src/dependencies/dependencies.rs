/// Imports
use crate::{
    config::config::{self, PackageConfig, PackageType},
    dependencies::dependencies,
    errors::PackageError,
};
use camino::Utf8PathBuf;
use console::style;
use git2::Repository;
use log::info;
use oil_common::bail;
use petgraph::{Direction, prelude::DiGraphMap};
use std::collections::{HashMap, HashSet};
use url::Url;

/// Finds cycle in a graph
fn find_cycle<'dep>(
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
        if find_cycle(origin, node, graph, path, done) {
            path.push(node);
            return true;
        }
    }
    false
}

/// Toposorts dependencies graph
fn toposort<'s>(deps: HashMap<&'s String, Vec<&'s String>>) -> Vec<&'s String> {
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
            if find_cycle(origin, origin, &deps_graph, &mut path, &mut HashSet::new()) {
                path.reverse();
                bail!(PackageError::FoundDependenciesCycle {
                    a: match path.get(0) {
                        Some(some) => (*some).clone(),
                        None => bail!(PackageError::CyclePathHasWrongLength { len: path.len() }),
                    },
                    b: match path.get(1) {
                        Some(some) => (*some).clone(),
                        None => bail!(PackageError::CyclePathHasWrongLength { len: path.len() }),
                    }
                })
            } else {
                bail!(PackageError::FailedToFindDependenciesCycle)
            }
        }
    }
}

/// Url to package name
///
// https://github.com/oil-rs/std -> std
// https://org.gittea.com/repo -> repo
// ...
pub fn url_to_pkg_name(url: &String) -> String {
    match Url::parse(url) {
        Ok(ok) => match ok.path_segments().and_then(|segments| segments.last()) {
            Some(segment) => match segment.strip_suffix(".git") {
                Some(name) => name.to_string(),
                None => bail!(PackageError::InvalidUrl { url: url.clone() }),
            },
            None => bail!(PackageError::InvalidUrl { url: url.clone() }),
        },
        Err(_) => bail!(PackageError::InvalidUrl { url: url.clone() }),
    }
}

/// Download dependency to cache,
/// If not already downloaded
///
/// Returns path to package and
/// package name
///
pub fn download<'s>(url: &'s String, cache: Utf8PathBuf) -> (Utf8PathBuf, String) {
    info!("Trying to download repository {url} to {cache}.");
    let package_name = url_to_pkg_name(url);
    let mut path = cache.clone();
    path.push(&package_name);
    // Checking already downloaded
    if path.exists() {
        info!("Repository {url} is already downloaded, skipping.")
    }
    // If not, downloading
    else {
        println!(
            "   {} Downloading: {package_name} from {url} ...",
            style("[🔗]").bold().bright().green()
        );
        match Url::parse(url) {
            Ok(_) => match Repository::clone(url, path.clone()) {
                Err(_) => bail!(PackageError::FailedToCloneRepo { url: url.clone() }),
                Ok(_) => {
                    info!("Repository from {url} download successfully.");
                }
            },
            Err(_) => bail!(PackageError::InvalidUrl { url: url.clone() }),
        }
        println!(
            "   {} Repository {package_name} downloaded successfully.",
            style("[✓]").bold().green()
        );
    }
    info!("Crawled name {package_name} from {url}.");
    (path, package_name)
}

/// Resolves packages,
/// returns recursivly found hash map
///
/// * cache -- .cache path
/// * solved -- already solved packages
/// * name -- package name
/// * config -- package config
///
fn resolve_packages<'cfg, 'solved>(
    cache: Utf8PathBuf,
    solved: &'solved mut HashMap<String, Vec<String>>,
    name: String,
    config: &'cfg PackageConfig,
) -> &'solved mut HashMap<String, Vec<String>> {
    // If already solved
    if solved.contains_key(&name) {
        solved
    }
    // If not
    else {
        info!("Resolving packages that {name} depends on.");
        // Inserting vector
        solved.insert(name.clone(), Vec::new());
        // Dependencies
        for dependency in &config.dependencies {
            // Downloading dependency if not already downloaded
            let pkg = dependencies::download(&dependency, cache.clone());
            let pkg_path = pkg.0;
            let pkg_name = pkg.1;
            let pkg_config = config::retrieve_config(&pkg_path);
            info!("+ Found dependency {} of {name}", &pkg_name);
            // Checking it's an `lib` pkg
            match pkg_config.pkg.pkg {
                PackageType::Lib => {
                    // Adding dependency
                    match solved.get_mut(&name) {
                        Some(vector) => vector.push(pkg_name.clone()),
                        None => bail!(PackageError::NoSolvedKeyFound { key: name }),
                    }
                    // Resolving dependency packages
                    resolve_packages(cache.clone(), solved, pkg_name, &pkg_config.pkg);
                }
                PackageType::App => bail!(PackageError::UseOfAppPackageAsDependency {
                    name: name,
                    path: pkg_path
                }),
            }
        }
        solved
    }
}

/// Solves dependencies,
///
/// returns toposorted vector
/// of packages
pub fn solve(cache: Utf8PathBuf, name: String, config: &PackageConfig) -> Vec<String> {
    // Solved packages
    let packages = resolve_packages(cache, &mut HashMap::new(), name, &config).to_owned();
    // Toposorting
    toposort(
        packages
            .iter()
            .map(|(k, v)| (k, v.iter().map(|e| e).collect()))
            .collect::<HashMap<&String, Vec<&String>>>(),
    )
    .iter()
    .map(|s| (*s).clone())
    .collect()
}
