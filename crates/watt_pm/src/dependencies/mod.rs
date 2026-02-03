/// Imports
use crate::{
    config::{self, PackageConfig, PackageDependency, PackageType},
    errors::PackageError,
    url::{path_to_pkg_name, url_to_pkg_name},
};
use camino::Utf8PathBuf;
use console::style;
use git2::Repository;
use petgraph::{Direction, prelude::DiGraphMap};
use std::collections::{HashMap, HashSet};
use tracing::{debug, info};
use url::Url;
use watt_common::bail;

/// Represents package
#[derive(Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Debug)]
pub struct Package {
    /// Package name
    pub name: String,
    /// Package path
    pub path: Utf8PathBuf,
}

/// Finds cycle in a graph
fn find_cycle<'dep>(
    origin: &'dep Package,
    parent: &'dep Package,
    graph: &petgraph::prelude::DiGraphMap<&'dep Package, ()>,
    path: &mut Vec<&'dep String>,
    done: &mut HashSet<&'dep Package>,
) -> bool {
    done.insert(parent);
    for node in graph.neighbors_directed(parent, Direction::Outgoing) {
        if node == origin {
            path.push(&node.name);
            return true;
        }
        if done.contains(&node) {
            continue;
        }
        if find_cycle(origin, node, graph, path, done) {
            path.push(&node.name);
            return true;
        }
    }
    false
}

/// Toposorts dependencies graph
fn toposort<'s>(deps: HashMap<&'s Package, Vec<&'s Package>>) -> Vec<&'s Package> {
    // Creating graph for toposorting
    let mut deps_graph: DiGraphMap<&Package, ()> =
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
                    a: match path.first() {
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

/// Download dependency to cache,
/// If not already downloaded
///
/// Returns path to package and
/// package name
///
pub fn download(url: &String, cache: Utf8PathBuf) -> Package {
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
            Ok(_) => match Repository::clone(url, &path) {
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
    Package {
        name: package_name,
        path,
    }
}

/// Resolves packages,
/// returns hash map of recursively solved modules.
///
/// # Parameters
/// - cache - `.cache` folder path
/// - solved - already solved packages
/// - package - package
/// - config - package config
///
fn resolve_packages<'solved>(
    cache: &Utf8PathBuf,
    solved: &'solved mut HashMap<Package, Vec<Package>>,
    package: Package,
    config: &PackageConfig,
) -> &'solved mut HashMap<Package, Vec<Package>> {
    // If already solved
    if solved.contains_key(&package) {
        solved
    }
    // If not
    else {
        info!("Resolving packages that {package:?} depends on.");
        debug!("Dependencies: {:?}", &config.dependencies);
        
        // Inserting vector
        solved.insert(package.clone(), Vec::new());
        // Dependencies
        for dependency in &config.dependencies {
            // Matching dependency
            match dependency {
                // Local dependency
                PackageDependency::Local { path } => {
                    // Retrieving dependency config
                    let path = Utf8PathBuf::from(path);
                    let pkg = Package {
                        name: path_to_pkg_name(&path),
                        path: path.clone(),
                    };
                    let pkg_config = config::retrieve_config(&path);
                    info!("+ Found local dependency {} of {pkg:?}", &package.name);
                    // Checking it's an `lib` pkg
                    match pkg_config.pkg.pkg {
                        PackageType::Lib => {
                            // Adding dependency
                            match solved.get_mut(&package) {
                                Some(vector) => vector.push(pkg.clone()),
                                None => bail!(PackageError::NoSolvedKeyFound { key: pkg.name }),
                            }
                            // Resolving dependency packages
                            resolve_packages(cache, solved, pkg, &pkg_config.pkg);
                        }
                        PackageType::App => bail!(PackageError::UseOfAppPackageAsDependency {
                            name: pkg.name,
                            path
                        }),
                    }
                }
                PackageDependency::Git(dependency) => {
                    // Downloading dependency if not already downloaded
                    let pkg = download(dependency, cache.clone());
                    let path = &pkg.path;
                    let pkg_config = config::retrieve_config(path);
                    info!("+ Found git dependency {} of {pkg:?}", &package.name);
                    // Checking it's an `lib` pkg
                    match pkg_config.pkg.pkg {
                        PackageType::Lib => {
                            // Adding dependency
                            match solved.get_mut(&package) {
                                Some(vector) => vector.push(pkg.clone()),
                                None => bail!(PackageError::NoSolvedKeyFound { key: pkg.name }),
                            }
                            // Resolving dependency packages
                            resolve_packages(cache, solved, pkg, &pkg_config.pkg);
                        }
                        PackageType::App => bail!(PackageError::UseOfAppPackageAsDependency {
                            name: pkg.name,
                            path: path.clone()
                        }),
                    }
                }
            }
        }
        solved
    }
}

/// Solves dependencies,
///
/// returns toposorted vector
/// of packages
pub fn solve(cache: Utf8PathBuf, pkg: Package, config: &PackageConfig) -> Vec<Package> {
    // Solved packages
    let packages = resolve_packages(&cache, &mut HashMap::new(), pkg, config).to_owned();
    // Toposorting
    toposort(
        packages
            .iter()
            .map(|(k, v)| (k, v.iter().collect()))
            .collect::<HashMap<&Package, Vec<&Package>>>(),
    )
    .iter()
    .map(|s| (*s).clone())
    .collect()
}
