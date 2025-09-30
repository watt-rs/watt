/// Modules
pub(crate) mod commands;
pub(crate) mod errors;
pub(crate) mod oil;

/// Imports
use clap::{Parser, Subcommand};

/*
 * Cli
 */

/// Cli itself
#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: SubCommand,
}

/// Subcommands
#[derive(Subcommand)]
enum SubCommand {
    /// Adds package from url
    Add { url: String },
    /// Removes package by name
    Remove { url: String },
    /// Runs project
    Run,
    /// Compiles project
    Compile,
    /// Creates new project
    New { name: String },
    /// Clears cache of packages
    Clean,
    /// Initializes new project in current folder
    Init,
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
