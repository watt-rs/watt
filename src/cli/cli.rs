// imports
use clap::{Parser};
use std::path::PathBuf;
use crate::executor::executor;

/// Cli clap parser
#[derive(Parser)]
struct CLI {
    #[arg(value_name = "file")]
    file: PathBuf,

    #[arg(long = "gc-threshold", value_name = "gc_threshold")]
    gc_threshold: Option<usize>,

    #[arg(long = "gc-debug", default_value_t=false)]
    gc_debug: bool,

    #[arg(long = "ast-debug", default_value_t=false)]
    ast_debug: bool,

    #[arg(long = "opcodes-debug", default_value_t=false)]
    opcodes_debug: bool,

    #[arg(long = "lexer-debug", default_value_t=false)]
    lexer_debug: bool,

    #[arg(long = "parse-bench", default_value_t=false)]
    parse_bench: bool,

    #[arg(long = "compile-bench", default_value_t=false)]
    compile_bench: bool,

    #[arg(long = "lexer-bench", default_value_t=false)]
    lexer_bench: bool,

    #[arg(long = "runtime-bench", default_value_t=false)]
    runtime_bench: bool,
}

/// Run cli
pub unsafe fn cli() {
    // parsing args
    let args = CLI::parse();

    // run executor with parsed args
    executor::run(
        args.file,
        args.gc_threshold,
        args.gc_debug,
        args.lexer_debug,
        args.ast_debug,
        args.opcodes_debug,
        args.lexer_bench,
        args.parse_bench,
        args.compile_bench,
        args.runtime_bench
    )
}
