/// CLI interface

// Modules
pub(crate) mod commands;
pub(crate) mod errors;
pub(crate) mod log;

// Imports
use crate::commands::{init, run, new};
use clap::{Parser, Subcommand};
use watt_pm::config::PackageType;

/// CLI itself
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
        runtime: Option<String>,
    },
    /// Analyzes project for compile-time errors.
    Analyze,
    /// Compiles project
    Compile,
    /// Creates new project
    New {
    	name: String,
    	
    	#[arg(value_enum)]
        package_type: Option<PackageType>,
    },
    /// Clears cache of packages
    Clean,
    /// Initializes new project in current folder
    Init {
    	#[arg(value_enum)]
        package_type: Option<PackageType>,
    },
}

/// Cli commands handler
pub fn cli() {
    // Parsing arguments
    match Cli::parse().command {
        SubCommand::Add { url: _ } => todo!(),
        SubCommand::Remove { url: _ } => todo!(),
        SubCommand::Run { runtime } => run::execute(runtime),
        SubCommand::Analyze => todo!(),
        SubCommand::Compile => todo!(),
        SubCommand::New { name, package_type } => new::execute(&name, package_type),
        SubCommand::Clean => todo!(),
        SubCommand::Init { package_type } => init::execute(package_type),
    }
}

/// Main function
fn main() {
    // Initializing logging
    log::init();
    // Cli
    cli();
}
