mod errors;
mod address;
mod colors;
mod lexer;
mod ast;
mod visitor;
mod parser;
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
        "test.wt".to_string()
    );
    match (lexer.lex()) {
        Ok(tokens) => {
            println!("Tokens: {:?}", tokens);
        }
        Err(err) => {
            err.print();
        }
    }
}