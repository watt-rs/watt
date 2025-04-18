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
use crate::lexer::address::Address;
use crate::vm::bytecode::{Chunk, Opcode};
use crate::vm::values::Value;
use crate::vm::vm::Vm;

// main
fn main() {
    /*
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
     */
    let mut vm = Vm::new();
    vm.run(Chunk::new(
        vec![
            Opcode::Push { addr: Address::new(0, "test".to_string()), value: Value::Integer(1) },
            Opcode::Push { addr: Address::new(0, "test".to_string()), value: Value::Integer(3) },
            Opcode::Bin { addr: Address::new(0, "test".to_string()), op: "+".to_string() },
        ]
    ));
    println!("{:?}", vm.pop(Address::new(0, "test".to_string())).unwrap());
}