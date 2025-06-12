// импорты
use clap::{Parser};
use std::path::PathBuf;
use crate::executor::executor;

// инструмент командной строки
#[derive(Parser)]
struct CLI {
    #[arg(value_name = "file")]
    file: PathBuf,

    #[arg(long = "gc-threshold", value_name = "gc_threshold")]
    gc_threshold: Option<usize>,

    #[arg(long = "gc-debug", value_name = "gc_debug")]
    gc_debug: Option<bool>,

    #[arg(long = "ast-debug", value_name = "ast_debug")]
    ast_debug: Option<bool>,

    #[arg(long = "opcodes-debug", value_name = "opcodes_debug")]
    opcodes_debug: Option<bool>,

    #[arg(long = "lexer-debug", value_name = "lexer_debug")]
    lexer_debug: Option<bool>,
}

pub unsafe fn cli() {
    // аргументы
    let args = CLI::parse();

    // запускаем
    executor::run(
        args.file,
        args.gc_threshold,
        args.gc_debug,
        args.lexer_debug,
        args.ast_debug,
        args.opcodes_debug,
    )
}