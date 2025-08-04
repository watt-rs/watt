// fixed 24 edition warnings
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(clippy::module_inception)]
#![allow(clippy::too_many_arguments)]
#![allow(dangerous_implicit_autorefs)]

// imports
use std::{fs, path::PathBuf};
use watt_analyze::analyzer::Analyzer;
use watt_ast::ast::Node;
use watt_common::{address::Address, error, errors::Error};
use watt_gen::generator::{BytecodeGenerator, GeneratorResult};
use watt_lex::{lexer::Lexer, tokens::Token};
use watt_parse::parser::Parser;
use watt_vm::{
    vm::{VM},
};

/// Reading file
///
/// raises error if path is not exists,
/// or file can not be read.
///
pub fn read_file(addr: Option<Address>, path: &PathBuf) -> String {
    // if path doesn't exist, we take the directory path of our program that imports the file
    let path: PathBuf = {
        if path.exists() {
            path.to_owned()
        } else if let Some(address) = &addr
            && let Some(file_path) = &address.file
        {
            match file_path.parent() {
                None => {
                    error!(Error::own_text(
                        address.to_owned(),
                        format!("file not found: {path:?}"),
                        "check file existence."
                    ))
                }
                Some(parent) => {
                    let mut result = parent.to_path_buf();
                    result.push(path);
                    if result.exists() {
                        result
                    } else {
                        error!(Error::own_text(
                            address.to_owned(),
                            format!("file not found: {path:?}"),
                            "check file existence."
                        ))
                    }
                }
            }
        } else {
            crash(format!("file not found: {path:?}"))
        }
    };

    // reading file
    if path.exists() {
        if let Ok(result) = fs::read_to_string(&path) {
            result
        } else if let Some(address) = addr {
            error!(Error::own_text(
                address,
                format!("io error with file: {path:?}"),
                "check file existence"
            ));
        } else {
            crash(format!("file not found: {path:?}"));
        }
    } else {
        panic!(
            "file not exists: {path:?} after checking of existence. report this error to the developer."
        )
    }
}

/// Runs code from a file
///
/// # Run args
///
/// * `gc_threshold`: garbage collector threshold
/// * `gc_debug`: on/off garbage collector debug
/// * `lexer_debug`: on/off lexer debug
/// * `ast_debug`: on/off ast debug
/// * `opcodes_debug`: on/of opcodes debug
/// * `lexer_bench`: on/off lexer benchmark
/// * `parser_bench`: on/off parser benchmark
/// * `compile_bench`: on/off compile benchmark
/// * `runtime_bench`: on/off runtime benchmark
///
#[allow(unused_qualifications)]
pub unsafe fn run(
    path: PathBuf,
    gc_threshold: Option<usize>,
    gc_threshold_grow_factor: Option<usize>,
    gc_debug: bool,
    lexer_debug: bool,
    ast_debug: bool,
    opcodes_debug: bool,
    lexer_bench: bool,
    parser_bench: bool,
    compile_bench: bool,
    runtime_bench: bool,
) {
    // reading file
    let code = read_file(Option::None, &path);

    // lexing
    let tokens = lex(
        &path,
        &code.chars().collect::<Vec<char>>(),
        lexer_debug,
        lexer_bench,
    );

    // parsing
    let ast = parse(&path, tokens.unwrap(), ast_debug, parser_bench);

    // analyzing
    let analyzed = analyze(ast);

    // compiling
    let compiled = compile(analyzed, opcodes_debug, compile_bench);

    // run compiled opcodes chunk with vm
    run_vm(
        compiled,
        runtime_bench,
    );
}

/// Crashes program with text
pub fn crash(reason: String) -> ! {
    println!("{reason}");
    std::process::exit(1);
}

/// Lexing source code
/// Provides tokens on the exhaust
pub fn lex(file_path: &PathBuf, code: &[char], debug: bool, bench: bool) -> Option<Vec<Token>> {
    // benchmark
    let start = std::time::Instant::now();

    // lexing
    let tokens = Lexer::new(code, file_path).lex();

    // benchmark end
    if bench {
        let duration = start.elapsed().as_nanos();
        println!(
            "benchmark 'lexer', elapsed {}",
            duration as f64 / 1_000_000f64
        );
    }

    // debug
    if debug {
        println!("tokens debug: ");
        println!("{tokens:?}");
    }

    Some(tokens)
}

/// Parsing
/// Provides AST node on the exhaust
pub fn parse(file_path: &PathBuf, tokens: Vec<Token>, debug: bool, bench: bool) -> Node {
    // benchmark
    let start = std::time::Instant::now();

    // building ast
    let ast = Parser::new(tokens, file_path).parse();

    // benchmark end
    if bench {
        let duration = start.elapsed().as_nanos();
        println!(
            "benchmark 'parse', elapsed {}",
            duration as f64 / 1_000_000f64
        );
    }

    // debug
    if debug {
        println!("ast debug: ");
        println!("{ast:?}");
    }

    // returning ast
    return ast;
}

/// Semantic analyzer
/// Provides analyzed node on the exhaust
pub fn analyze(ast: Node) -> Node {
    Analyzer::new().analyze(&ast);
    ast
}

/// Compilation
/// Provides compiled chunk on the exhaust
pub unsafe fn compile(ast: Node, opcodes_debug: bool, bench: bool) -> GeneratorResult {
    // benchmark
    let start = std::time::Instant::now();

    // compile
    let compiled = BytecodeGenerator::new().generate(ast);

    // benchmark end
    if bench {
        let duration = start.elapsed().as_nanos();
        println!(
            "benchmark 'compile', elapsed {}",
            duration as f64 / 1_000_000f64
        );
    }

    // debug
    if opcodes_debug {
        println!("main module: ");
        for op in compiled.main.opcodes() {
            op.print(1);
        }
        println!("builtins module: ");
        for op in compiled.builtins.opcodes() {
            op.print(1);
        }
        println!("other modules: ");
        for id in compiled.modules.keys() {
            let module = compiled.modules.get(id).unwrap();
            println!("  {:?}:", module.path);
            for op in module.chunk.opcodes() {
                op.print(2);
            }
        }
    }

    compiled
}

/// Runs bytecode on the vm
///
/// * gc_threshold: garbage collector threshold
/// * gc_threshold_grow_factor: garbage collector threshold grow factor
#[allow(unused_qualifications)]
unsafe fn run_vm(
    bytecode: GeneratorResult,
    bench: bool,
) {
    // benchmark
    let start = std::time::Instant::now();

    // creating vm and running
    let mut vm = VM::new(
        bytecode.builtins,
        bytecode.modules,
    );

    // handling errors
    match vm.run_module(&bytecode.main) {
        Err(err) => error!(Error::own_text(
            Address::unknown(),
            format!("control flow leak: {err:?}"),
            "report this error to the developer."
        )),
        _ => {}
    }

    // benchmark end
    if bench {
        let duration = start.elapsed().as_nanos();
        println!(
            "benchmark 'runtime', elapsed {}",
            duration as f64 / 1_000_000f64
        );
    }
}
