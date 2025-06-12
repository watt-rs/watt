// объявление модулей
mod lexer;
mod compiler;
mod parser;
mod vm;
mod errors;
mod semantic;
mod cli;
mod executor;

// main
fn main() {
    unsafe { cli::cli::cli() }
}