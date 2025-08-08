/// Imports
use miette::NamedSource;
use oil_lex::lexer::Lexer;
use oil_parse::parser::Parser;
use std::{fs, path::PathBuf};

/// Runs code
#[allow(unused_variables)]
pub fn run(path: PathBuf, lex_debug: bool, parse_debug: bool) {
    // code
    let code = fs::read_to_string(&path).unwrap();
    let code_chars: Vec<char> = code.chars().collect();
    // named source
    let named_source = NamedSource::<String>::new("test", code);
    // lex
    let lexer = Lexer::new(&code_chars, &path, &named_source);
    let tokens = lexer.lex();
    // result
    println!("tokens:");
    println!("{:?}", tokens);
    // parse
    let mut parser = Parser::new(tokens, &named_source);
    let ast = parser.parse();
    // result
    println!("ast:");
    println!("{:?}", ast);
}
