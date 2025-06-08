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
use crate::errors::*;
use crate::compiler::visitor::CompileVisitor;
use crate::vm::*;

fn exec() -> Result<(), Error> {
    let code = String::from("
    fun fib(a) {
        if a <= 1 {
            return a
        } else {
            return fib(a - 1) + fib(a - 2)
        }
    }
    println(fib(15))");
    let file_name = String::from("main.rs");
    let tokens = Lexer::new(code, file_name.clone()).lex()?;
    println!("tokens: {:?}", tokens.clone());
    println!("...");
    let ast = Parser::new(tokens, file_name.clone(), "main".to_string()).parse()?;
    println!("ast: {:?}", ast.clone());
    println!("...");
    let opcodes = CompileVisitor::new().compile(ast)?;
    println!("opcodes: {:?}", opcodes.clone());
    println!("...");
    // println!("{:?}", frame);
    Ok(())
}

// main
fn main() {
    /*
    #[derive(Debug)]
    struct A {
        a: i32
    }
    unsafe {
        let int = memory::alloc_value(A{a:5});
        println!("integer: {:?} address = {:?}", *int, int);
        memory::free_value(int);
        println!("integer: {:?} address = {:?}", *int, int);
    }
     */
    if let Err(e) = exec() {
        e.print()
    }
}