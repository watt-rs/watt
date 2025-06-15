// объявление модулей
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