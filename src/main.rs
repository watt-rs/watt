/*
Main modules
 */
mod errors;
mod lexer;
mod colors;
mod compiler;
mod import;
mod parser;
mod vm;

// imports
use crate::lexer::lexer::Lexer;
use crate::parser::parser::Parser;

// main
fn main() {
    let mut lexer = Lexer::new(
        "fun main() { io.println('Hello, world!') }".to_string(),
        "test.gko".to_string()
    );
    match lexer.lex() {
        Ok(tokens) => {
            println!("Tokens: {:?}", tokens.clone());
            let mut parser = Parser::new(
                tokens,
                "test.gko".to_string(),
                "test".to_string()
            );
            match parser.parse() {
                Ok(ast) => {
                    println!("AST: {:?}", ast);
                }
                Err(err) => {
                    err.print()
                }
            }
        }
        Err(err) => {
            err.print();
        }
    }
}