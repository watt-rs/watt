/// Moduled
mod errors;

/// Imports
use camino::Utf8PathBuf;
use ecow::EcoString;
use miette::NamedSource;
use oil_analyze::untyped_ir::{self, untyped_ir::UntypedModule};
use oil_common::bail;
use oil_lex::lexer::Lexer;
use oil_parse::parser::Parser;
use std::fs;
use crate::errors::CompileError;

/// Module loader
pub struct ModuleLoader {
    /// Module path
    pub file: Utf8PathBuf,
}
/// Module loader implementation
impl ModuleLoader {
    /// To module name
    pub fn to_module_name(path: Utf8PathBuf) -> EcoString {
        // retrieving module name
        let file_name = path.file_name().unwrap_or_else(|| bail!(CompileError::PathIsNotAFile {
            unexpected: path
        })).strip_prefix("/src");
        
        // 
    }

    /// Loads module, returns untyped module.
    pub fn load(path: Utf8PathBuf) -> UntypedModule {
        // path to module name
        let module_name = Self::to_module_name(path);
        // reading code
        let code = fs::read_to_string(&path).unwrap();
        let code_chars: Vec<char> = code.chars().collect();
        // creating named source for miette
        let named_source = NamedSource::<String>::new(module_name, code);
        // lexing
        let lexer = Lexer::new(&code_chars, &named_source);
        let tokens = lexer.lex();
        // parsing
        let mut parser = Parser::new(tokens, &named_source);
        let tree = parser.parse();
        // untyped ir
        let untyped_ir = untyped_ir::lowering::tree_to_ir(&named_source, tree);
        return untyped_ir;
    }
}

/// Package compiler
pub struct PackageCompiler {
    /// Package path
    pub package: Utf8PathBuf,
    /// Local main path `$package/path/to/file.oil`
    pub main: Utf8PathBuf,
}
/// Package compiler implementation
impl PackageCompiler {}
