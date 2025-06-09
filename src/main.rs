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

use std::time::{SystemTime, UNIX_EPOCH};
// imports
use crate::lexer::lexer::Lexer;
use crate::parser::parser::Parser;
use crate::errors::*;
use crate::compiler::visitor::CompileVisitor;
use crate::lexer::address::Address;
use crate::vm::*;
use crate::vm::flow::ControlFlow;
use crate::vm::table::Table;
use crate::vm::values::Value;
use crate::vm::vm::VM;

unsafe fn exec() -> Result<(), Error> {
    let code = String::from("
    /*
    unit test_unit {
        a := 4.7
    }
    type Bird(speed) {
        fun fly() {
            println(speed)
        }
    }
    bird := new Bird(3.7)
    bird.fly()
    bird.speed = test_unit.a
    bird.fly()
    */
    /*
    i := 0
    while i < 1000000 {
        println(i)
        i += 1
    }
    */
    /*
    type Gecko {
        fun say_hello(name) {
            println('Hello, ' + name)
        }
    }

    gecko := new Gecko()
    gecko.say_hello('Vyacheslav')
    println(gecko)
    println(gecko.say_hello)
    */
    /*
    fun factorial(n) {
        f := 1
        while n > 1 {
            f *= n
            n -= 1
        }
        return f
    }
    println(factorial(15))
    */
    /*
    a := 5
    type Doggy {
    }
    doggy := new Doggy()
    doggy.a = 7
    println(a)
    */
    fun first {
        a := 5
        fun second {
            b := 7
            println(a + b)
        }
        return second
    }
    fn := first()
    fn()");
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
    let start = SystemTime::now();
    let start_millis = start.duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos();
    println!("runtime: ");
    let mut vm = VM::new()?;
    if let Err(e) = vm.run(opcodes, vm.globals) {
        if let ControlFlow::Error(error) = e {
            return Err(error);
        } else {
            return Err(Error::new(
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