/// Imports
use crate::errors::PackageError;
use camino::Utf8PathBuf;
use ecow::EcoString;
use git2::Repository;
use log::info;
use oil_common::bail;
use petgraph::{Direction, prelude::DiGraphMap};
use std::collections::{HashMap, HashSet};
use url::Url;

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
        if find_cycle(origin, node, graph, path, done) {
            path.push(node);
            return true;
        }
    }
    false
}

/// Toposorts dependencies graph
fn toposort<'s>(deps: HashMap<&'s EcoString, Vec<&'s EcoString>>) -> Vec<&'s EcoString> {
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
    match Url::parse(url) {
        Ok(_) => match Repository::clone(url, path.clone()) {
            Err(error) => match error.code() {
                git2::ErrorCode::Exists => {
                    info!("Repository {url} is already downloaded, skipping.")
                }
                _ => {
                    bail!(PackageError::FailedToCloneRepo { url: url.clone() })
                }
            },
            Ok(_) => {
                info!("Repository from {url} download successfully.");
            }
        },
        Err(_) => bail!(PackageError::InvalidUrl { url: url.clone() }),
    }
    let mut path = cache.clone();
    path.push(package_name.as_str());
    info!("Crawled name {package_name} from {url}.");
    (path, package_name)
}
