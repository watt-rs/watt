// Lints
// Todo: remove when `https://github.com/rust-lang/rust/issues/147648` will be resolved.
#![allow(unused_assignments)]

// Modules
pub mod check;
pub mod cx;
pub mod errors;
pub mod ex;
pub mod inference;
pub mod pretty;
pub mod resolve;
pub mod typ;
pub mod warnings;
