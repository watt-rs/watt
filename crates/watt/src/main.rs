/// Modules
pub(crate) mod commands;
pub(crate) mod errors;
pub(crate) mod log;

/// Imports
use crate::commands::{init, run};
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
    Run {
        #[arg(value_parser = ["deno", "bun", "node"])]
        rt: Option<String>,
    },
    /// Analyzes project for compile-time errors.
    Analyze,
    /// Compiles project
    Compile,
    /// Creates new project
    New { name: String },
    /// Clears cache of packages
    Clean,
    /// Initializes new project in current folder
    Init {
        #[arg(value_parser = ["app", "lib"])]
        ty: Option<String>,
    },
}

/// Cli commands handler
pub fn cli() {
    // Parsing arguments
    match Cli::parse().command {
        SubCommand::Add { url: _ } => todo!(),
        SubCommand::Remove { url: _ } => todo!(),
        SubCommand::Run { rt } => run::execute(rt),
        SubCommand::Analyze => todo!(),
        SubCommand::Compile => todo!(),
        SubCommand::New { name: _ } => todo!(),
        SubCommand::Clean => todo!(),
        SubCommand::Init { ty } => init::execute(ty),
    }
}

/// Main function
fn main() {
    // Initializing logging
    log::init();
    // Cli
    cli();
}
