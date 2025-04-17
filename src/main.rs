mod errors;
mod address;
mod colors;
mod lexer;
mod ast;
mod visitor;
mod parser;
mod import;
/*
fn a(a : bool) -> Result<String, String> {
    if a {
        Result::Ok(String::from("Hello, world!"))
    } else {
        Result::Err(String::from("Error"))
    }
}

fn main() {
    match a(true) {
        Ok(s) => println!("{}", s),
        Err(s) => println!("{}", s),
    }
}
*/

fn main() {
    let mut lexer = lexer::Lexer::new(
        "fun main() { io.println('Hello, world!') }".to_string(),
        "test.gko".to_string()
    );
    match lexer.lex() {
        Ok(tokens) => {
            println!("Tokens: {:?}", tokens.clone());
            let mut parser = parser::Parser::new(
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