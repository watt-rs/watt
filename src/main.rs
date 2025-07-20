// fixed 24 edition warnings
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(clippy::module_inception)]
#![allow(clippy::too_many_arguments)]
#![allow(dangerous_implicit_autorefs)]

// modules
mod cli;
mod compiler;
mod errors;
mod executor;
mod lexer;
mod parser;
mod resolver;
mod semantic;
mod vm;

// main
fn main() {
    unsafe { cli::cli::cli() }
}
