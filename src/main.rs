// объявление модулей
mod lexer;
mod compiler;
mod parser;
mod vm;
mod errors;
mod semantic;

// импорты
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::lexer::lexer::Lexer;
use crate::parser::parser::Parser;
use crate::errors::*;
use crate::compiler::visitor::CompileVisitor;
use crate::errors::errors::{Error, ErrorType};
use crate::lexer::address::Address;
use crate::semantic::analyzer::Analyzer;
use crate::vm::flow::ControlFlow;
use crate::vm::table::Table;
use crate::vm::threads::threads::Threads;
use crate::vm::values::Value;
use crate::vm::vm::{VmSettings, VM};
use crate::vm::memory::memory;
use crate::vm::statics::statics;
use crate::vm::statics::statics::VM_PTR;

// исполнения
#[allow(unused_mut)]
#[allow(unused_qualifications)]
unsafe fn exec() -> Result<(), Error> {
    let code = match fs::read_to_string("./src/test.wt") {
        Ok(code) => code,
        Err(e) => {
            return Err(Error::new(
                ErrorType::Parsing,
                Address::new(0, "internal".to_string()),
                format!("io error: {}", e.to_string()),
                "check file test.wt existence.".to_string(),
            ));
        }
    };
    let file_name = String::from("main.rs");
    let tokens = Lexer::new(code, file_name.clone()).lex()?;
    println!("tokens: {:?}", tokens.clone());
    println!("...");
    let ast = Parser::new(tokens, file_name.clone(), "main".to_string()).parse()?;
    println!("ast: {:?}", ast.clone());
    println!("...");
    println!("analyzing...");
    println!("...");
    let mut analyzer = Analyzer::new();
    analyzer.analyze(ast.clone())?;
    let opcodes = CompileVisitor::new().compile(ast)?;
    println!("opcodes: {:?}", opcodes.clone());
    println!("...");
    let start = SystemTime::now();
    let start_millis = start.duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos();
    println!("runtime: ");
    statics::THREADS_PTR = Option::Some(memory::alloc_value(Threads::new()));
    let mut vm = VM::new(VmSettings::new(
        100,
        true
    ), statics::THREADS_PTR.unwrap())?;
    VM_PTR = Option::Some(memory::alloc_value(vm));
    if let Err(e) = (*VM_PTR.unwrap()).run(opcodes, (*VM_PTR.unwrap() ).globals) {
        return if let ControlFlow::Error(error) = e {
            Err(error)
        } else {
            Err(Error::new(
                ErrorType::Runtime,
                Address::new(0, "internal".to_string()),
                format!("flow leak: {:?}", e),
                "check your code".to_string()
            ))
        }
    }
    let end = SystemTime::now();
    let end_millis = end.duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos();
    // ждём окончания потоков
    (*(*VM_PTR.unwrap()).threads).wait_finish();
    print!("wait finished");
    // выводим время
    println!("time: {}ms", ((end_millis-start_millis) as f64)/1_000_000.0);
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
    unsafe {
        if let Err(e) = exec() {
            e.print()
        }
    }
    /*
    unsafe {
        if let Err(e) = test_tables() {
            e.print()
        }
    }
     */
}

unsafe fn test_tables() -> Result<(), Error> {
    let mut table = Table::new();
    let addr = Address::new(0, "test".to_string());
    table.set_root(memory::alloc_value(Table::new()));
    (*table.root).closure = memory::alloc_value(Table::new());
    let str = memory::alloc_value("Hello, world!".to_string());
    let str2 = memory::alloc_value("Hello, world 2!".to_string());
    (*(*table.root).closure).define(addr.clone(), "hello".to_string(), Value::String(str))?;
    table.define(addr.clone(), "hello".to_string(), Value::String(str2))?;
    let val = table.lookup(addr.clone(), "hello".to_string())?;
    match val {
        Value::String(s) => {
            println!("val: {:?}", *s);
        }
        _ => {return  Ok(())}
    }
    Ok(())
}