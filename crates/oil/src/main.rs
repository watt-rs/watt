/// Modules
mod oil;

/// Imports
use clap::{Arg, ArgAction};

/// Run cli
pub fn cli() {
    // Command-line parser
    let parser = clap::Command::new("oil")
        .author("Oil developers.")
        .about("Oil compiler.")
        .arg(
            Arg::new("ast-debug")
                .long("ast-debug")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("lexer-debug")
                .long("lexer-debug")
                .action(ArgAction::SetTrue),
        )
        .arg(Arg::new("file").required(true))
        .arg(Arg::new("args").action(ArgAction::Append));

    let matches = parser.get_matches();
    let file = matches.get_one::<String>("file").unwrap();

    // run executor with parsed args
    oil::run(
        file.into(),
        matches.get_flag("lexer-debug"),
        matches.get_flag("ast-debug"),
    )
}

fn main() {
    cli();
}
