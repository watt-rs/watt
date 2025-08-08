/// Imports
use std::{fs, path::PathBuf};

use oil_common::colors;
use oil_lex::lexer::Lexer;
use oil_parse::parser::Parser;

/// Runs code
#[allow(unused_variables)]
pub fn run(path: PathBuf, lex_debug: bool, parse_debug: bool) {
    // lex
    let code: Vec<char> = fs::read_to_string(&path).unwrap().chars().collect();
    let lexer = Lexer::new(&code, &path);
    let tokens = lexer.lex();
    // result
    println!("{}tokens:{}", colors::CyanColor, colors::WhiteColor);
    println!("{}{:?}", colors::GreenColor, tokens);
    // parse
    let mut parser = Parser::new(tokens, &path);
    let ast = parser.parse();
    // result
    println!("{}ast:{}", colors::CyanColor, colors::WhiteColor);
    println!("{}{:#?}", colors::GreenColor, ast);
}
