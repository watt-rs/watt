/// Imports
use camino::Utf8PathBuf;
use ecow::EcoString;
use miette::NamedSource;
use std::{collections::HashMap, sync::Arc};
use watt_ast::ast;
use watt_common::package::{DraftPackage, DraftPackageLints};
use watt_gen::gen_module;
use watt_lex::lexer::Lexer;
use watt_lint::lint::LintCx;
use watt_parse::parser::Parser;
use watt_typeck::{
    cx::{module::ModuleCx, package::PackageCx, root::RootCx},
    typ::cx::TyCx,
};

/// Test module name used for compilation
const TEST_MODULE_NAME: &str = "buggy";

/// Loads watt module
#[allow(dead_code)]
fn load_module(code: String, draft: &DraftPackage) -> ast::Module {
    // Reading code
    let code_chars: Vec<char> = code.chars().collect();
    // Creating named source for miette
    let named_source = Arc::new(NamedSource::<String>::new(TEST_MODULE_NAME, code));
    // Lexing
    let lexer = Lexer::new(&code_chars, &named_source);
    let tokens = lexer.lex();
    // Parsing
    let mut parser = Parser::new(tokens, &named_source);
    let ast = parser.parse();
    // Linting
    let linter = LintCx::new(draft, &ast);
    linter.lint();
    // Done
    ast
}

/// Compiles watt into js
#[allow(dead_code)]
pub(crate) fn generate_js(code: &str) -> String {
    // Draft package
    let draft_package = DraftPackage {
        path: Utf8PathBuf::new(),
        lints: DraftPackageLints {
            disabled: Vec::new(),
        },
    };
    let module_name = EcoString::from(TEST_MODULE_NAME);
    // Loaded module
    let module = load_module(code.to_string(), &draft_package);
    // Typechecking
    let mut tcx = TyCx::default();
    let mut root_cx = RootCx {
        modules: HashMap::new(),
    };
    let package_cx = PackageCx {
        draft: draft_package,
        root: &mut root_cx,
    };
    let mut module_cx = ModuleCx::new(&module, &module_name, &mut tcx, &package_cx);
    let _ = module_cx.analyze();
    // Generating code
    gen_module(&module_name, &module).to_file_string().unwrap()
}

/// Parses watt into ast
#[allow(dead_code)]
pub(crate) fn parse_into_ast(code: &str) -> ast::Module {
    // Draft package
    let draft_package = DraftPackage {
        path: Utf8PathBuf::new(),
        lints: DraftPackageLints {
            disabled: Vec::new(),
        },
    };
    // Loaded module
    let module = load_module(code.to_string(), &draft_package);
    module
}

/// Asserts javascript generation result.
#[macro_export]
macro_rules! assert_js {
    ($src:expr $(,)?) => {{
        let compiled = match std::panic::catch_unwind(|| $crate::js::generate_js($src)) {
            Ok(result) => result,
            Err(err) => {
                let panic_str = if let Some(s) = err.downcast_ref::<&str>() {
                    (*s).to_string()
                } else if let Some(s) = err.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "<failed to retrieve panic message>".to_string()
                };
                format!("{}", panic_str)
            }
        };
        let output = format!("Source code:\n{}\n\nGeneration result:\n{compiled}", $src);
        let re = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
        let cleaned = re.replace_all(&output, "").to_string();
        insta::assert_snapshot!(insta::internals::AutoName, cleaned, $src);
    }};
}

/// Asserts AST parsing result.
#[macro_export]
macro_rules! assert_ast {
    ($src:expr $(,)?) => {{
        let ast =
            match std::panic::catch_unwind(|| format!("{:#?}", $crate::js::parse_into_ast($src))) {
                Ok(result) => result,
                Err(err) => {
                    let panic_str = if let Some(s) = err.downcast_ref::<&str>() {
                        (*s).to_string()
                    } else if let Some(s) = err.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "<failed to retrieve panic message>".to_string()
                    };
                    format!("{}", panic_str)
                }
            };
        let output = format!("Source code:\n{}\n\nAst:\n{ast}", $src);
        let re = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
        let cleaned = re.replace_all(&output, "").to_string();
        insta::assert_snapshot!(insta::internals::AutoName, cleaned, $src);
    }};
}
