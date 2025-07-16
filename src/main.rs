// fixed 24 edition warnings
#![allow(unsafe_op_in_unsafe_fn)]

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
