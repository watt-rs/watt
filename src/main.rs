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

use std::thread;
use crate::compiler::visitor::CompileVisitor;
// imports
use crate::lexer::lexer::Lexer;
use crate::parser::parser::Parser;
use crate::errors::*;
use crate::lexer::address::Address;
use crate::vm::frames::Frame;
use crate::vm::utils::SyncCell;
use crate::vm::vm::{ControlFlow, Vm};

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
    // запуск
    let builder = thread::Builder::new().stack_size(32 * 1024 * 1024); // 32 МБ
    let handler = builder.spawn(|| {
        let mut vm = Vm::new();
        let address = Address::new(0, "main.rs".to_string());
        let mut frame = SyncCell::new(Frame::new());
        return match vm.run(opcodes, frame.clone()) {
            Ok(_) => Ok(()),
            Err(e) => {
                match e {
                    ControlFlow::Error(e) => Err(e),
                    _ => Err(Error::new(
                        ErrorType::Runtime,
                        address.clone(),
                        "flow leak.".to_string(),
                        "check your code.".to_string(),
                    )),
                }
            }
        };
    }).unwrap();
    handler.join().unwrap()?;
    // println!("{:?}", frame);
    Ok(())
}

// main
fn main() {
    /*
    let mut lexer = Lexer::new(
        "fun main() { io.println('Hello, world!') }".to_string(),
        "test.wt".to_string()
    );
    match lexer.lex() {
        Ok(tokens) => {
            println!("Tokens: {:?}", tokens.clone());
            let mut parser = Parser::new(
                tokens,
                "test.wt".to_string(),
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
    /*
    let mut vm = Vm::new();
    vm.run(Chunk::new(
        vec![
            Opcode::Push { addr: Address::new(0, "test".to_string()), value: Value::Integer(1) },
            Opcode::Push { addr: Address::new(0, "test".to_string()), value: Value::Integer(3) },
            Opcode::Bin { addr: Address::new(0, "test".to_string()), op: "+".to_string() },
        ]
    ));
    println!("{:?}", vm.pop(Address::new(0, "test".to_string())).unwrap());

     */
    /*
    let mut root_frame = Frame::new();
    let mut frame = None;
    let addr = Address::new(0, "example.wt".to_string());
    if let Err(e) = root_frame.define(addr.clone(), "hello".to_string(), Value::String("World".to_string())) {
        println!("{:?} err1: ", e)
    } else {
        let mut child_frame = Frame::new();
        child_frame.root = Option::Some(Arc::new(Mutex::new(root_frame)));
        let mut closure_frame = Frame::new();
        closure_frame.define(addr.clone(), "role".to_string(), Value::String("Perviy".to_string()));
        child_frame.closure = Option::Some(Arc::new(Reetram::new(closure_frame)));
        frame = Some(child_frame.clone());
        if let Err(e) = child_frame.set(addr.clone(), "hello".to_string(), Value::String("Petr!".to_string())) {
            println!("{:?} err2!", e)
        }
    }
    let unwrapped = frame.unwrap();
    // println!("{:?}", unwrapped);
    println!("{:?} is a result! And he is {:?}", unwrapped.lookup(addr.clone(), "hello".to_string()), unwrapped.lookup(addr, "role".to_string()));
     */
    if let Err(e) = exec() {
        e.print()
    }
}