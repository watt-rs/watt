// imports
use std::path::PathBuf;
use crate::executor::executor;
use clap::{Command,Arg,ArgAction};

/// Run cli
pub unsafe fn cli() {
    // Command-line parser
    let parser = clap::Command::new("watt")
        .author("Watt developers")
        .about("The Watt interpreter.")
        .arg(
            Arg::new("gc-debug")
                .long("gc-debug")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("ast-debug")
                .long("ast-debug")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("opcodes-debug")
                .long("opcodes-debug")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("lexer-debug")
                .long("lexer-debug")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("parse-bench")
                .long("parser-bench")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("compile-bench")
                .long("compile-bench")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("lexer-bench")
                .long("lexer-bench")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("runtime-bench")
                .long("runtime-bench")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("gc-threshold")
                .long("gc-threshold")
                .value_parser(clap::value_parser!(usize))
                .default_value("200"),
        )
        .arg(Arg::new("file").required(true));

    let matches = parser.get_matches();

    let file = matches.get_one::<String>("file").unwrap();

    // run executor with parsed args
    executor::run(
        file.into(),
        matches.get_one::<usize>("gc-threshold").copied(),
        matches.get_flag("gc-debug"),
        matches.get_flag("lexer-debug"),
        matches.get_flag("ast-debug"),
        matches.get_flag("opcodes-debug"),
        matches.get_flag("lexer-bench"),
        matches.get_flag("parse-bench"),
        matches.get_flag("compile-bench"),
        matches.get_flag("runtime-bench")
    )
}
