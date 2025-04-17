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

use crate::compiler::visitor::CompileVisitor;
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
            println!("--------------");
            match parser.parse() {
                Ok(ast) => {
                    println!("AST: {:?}", ast);
                    let mut compiler = CompileVisitor::new();
                    println!("--------------");
                    match compiler.compile(ast.clone()) {
                        Ok(opcodes) => {
                            println!("Opcodes: {:?}", opcodes)
                        }
                        Err(err) => {
                            err.print()
                        }
                    }
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