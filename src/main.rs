// fixed 24 edition warnings
#![allow(unsafe_op_in_unsafe_fn)]

// modules
mod lexer;
mod compiler;
mod parser;
mod vm;
mod errors;
mod semantic;
mod cli;
mod executor;
mod resolver;

// main
fn main() {
    unsafe { cli::cli::cli() }
}