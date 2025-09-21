/// Modules
pub(crate) mod commands;
pub(crate) mod config;
pub(crate) mod dependencies;
pub(crate) mod errors;
pub(crate) mod oil;

/// Imports
use clap::{Parser, Subcommand};

/*
 * Cli
 */

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: SubCommand,
}

#[derive(Subcommand)]
enum SubCommand {
    /// Adds package from url
    Add { url: String, package_name: String },
    /// Removes package by name
    Remove { package_name: String },
    /// Runs project
    Run,
    /// Builds porject
    Build,
    /// Creates new project
    New { name: String },
}

/// Cli commands handler
pub fn cli() {
    oil::run();
}

/// Main function
fn main() {
    // Initializing logging
    pretty_env_logger::init();
    // Cli
    cli();
}
