// импорты
use std::fs;
use std::path::PathBuf;
use cliclack::ProgressBar;
use crate::compiler::visitor::CompileVisitor;
use crate::error;
use crate::errors::errors::Error;
use crate::lexer::address::Address;
use crate::lexer::lexer::{Lexer, Token};
use crate::parser::ast::Node;
use crate::parser::parser::Parser;
use crate::semantic::analyzer::Analyzer;
use crate::vm::bytecode::Chunk;
use crate::vm::memory::memory;
use crate::vm::statics::statics;
use crate::vm::threads::threads::Threads;
use crate::vm::vm::{VmSettings, VM};

// запуск кода
pub unsafe fn run(
    path: PathBuf,
    gc_threshold: Option<usize>,
    gc_debug: Option<bool>,
    lexer_debug: Option<bool>,
    ast_debug: Option<bool>,
    opcodes_debug: Option<bool>,
) {
    // спиннер компиляции
    let spinner = start_spinner();
    // чтение файла
    let code = read_file(path.clone());
    // выключаем спиннер
    spinner.stop("running...");
    // имя файла
    let filename = path.file_name().unwrap().to_str().unwrap().to_string();
    // компиляция
    let tokens = lex(
        filename.clone(),
        code,
        lexer_debug.unwrap_or(false)
    );
    let ast = parse(
        filename,
        tokens.unwrap(),
        ast_debug.unwrap_or(false),
    );
    let analyzed = analyze(
        ast.unwrap()
    );
    let compiled = compile(
        analyzed,
        opcodes_debug.unwrap_or(false),
    );
    // запуск
    run_chunk(
        compiled,
        gc_threshold.unwrap_or(200),
        gc_debug.unwrap_or(false)
    );
}

// краш
fn crash(reason: String) {
    // крашим и выходим
    println!("{}", reason);
    std::process::exit(1);
}

// чтение файла
fn read_file(path: PathBuf) -> String {
    if path.exists() {
        if let Ok(result) = fs::read_to_string(path.clone()) {
            result
        } else {
            crash(
                format!(
                    "io error with file: {:?}",
                    path
                )
            );
            "".to_string()
        }
    }
    else {
        crash(
            format!(
                "file not found: {:?}",
                path
            )
        );
        "".to_string()
    }
}

// лексинг
fn lex(file_name: String, code: String, debug: bool) -> Option<Vec<Token>> {
    // сканнинг токенов
    let tokens = Lexer::new(
        code,
        file_name
    ).lex();
    // проверяем на дебаг
    if debug {
        println!("tokens debug: ");
        println!("{:?}", tokens.clone());
    }
    // возвращаем
    Some(tokens)
}


// парсинг
fn parse(file_name: String, tokens: Vec<Token>, debug: bool) -> Option<Node> {
    // стройка аст
    let raw_ast = Parser::new(
        tokens,
        file_name.clone(),
        file_name
    ).parse();
    // проверяем на ошибку
    if let Ok(ast) = raw_ast {
        // проверяем на дебаг
        if debug {
            println!("ast debug: ");
            println!("{:?}", ast.clone());
        }
        // возвращаем
        return Some(ast)
    } else if let Err(error) = raw_ast {
        // ошибка
        error!(error);
        // ничего не возвращаем
        return None
    };
    // паника
    panic!("result error in parsing. report to developer.")
}

// семантический анализ
fn analyze(ast: Node) -> Node {
    // анализ
    let analyzed = Analyzer::new().analyze(ast);
    // возвращаем
    analyzed
}

// компиляция
fn compile(ast: Node, opcodes_debug: bool) -> Chunk {
    // компилируем
    let compiled = CompileVisitor::new().compile(ast);
    // дебаг
    if opcodes_debug {
        println!("opcodes debug: ");
        println!("{:?}", compiled.clone());
    }
    // возвращаем
    compiled
}

// запуск
#[allow(unused_qualifications)]
unsafe fn run_chunk(chunk: Chunk, gc_threshold: usize, gc_debug: bool) {
    // потоки
    let threads = memory::alloc_value(Threads::new());
    // указатель
    statics::THREADS_PTR = Option::Some(threads);
    // вм
    let vm = memory::alloc_value(VM::new(VmSettings::new(
        gc_threshold,
        gc_debug,
    ), threads));
    // указатель
    statics::VM_PTR = Option::Some(vm);
    // запуск
    if let Err(e) = (*vm).run(chunk, (*vm).globals) {
        error!(Error::new(
            Address::new(
                0,
                0,
                "-".to_string(),
                "-".to_string()
            ),
            format!("control flow leak: {:?}", e),
            "report this error to the developer.".to_string()
        ));
    }
    // ожидаем завершения потоков
    (*(*vm).threads).wait_finish();
}

// спиннер компиляции
fn start_spinner() -> ProgressBar {
    // запускаем
    let bar = cliclack::spinner();
    bar.start("compilation...");
    bar
}