// Lints
// Todo: remove when `https://github.com/rust-lang/rust/issues/147648` will be resolved.
#![allow(unused_assignments)]

// Modules
pub mod compile;
pub mod config;
pub mod dependencies;
mod errors;
pub mod generate;
pub mod runtime;
pub mod url;
