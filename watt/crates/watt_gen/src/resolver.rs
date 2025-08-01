// imports
use core::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use watt_analyze::analyzer::Analyzer;
use watt_ast::{ast::Node, import::Import};
use watt_common::{
    address::Address,
    error,
    errors::Error,
    fs::{self, FileReadError, delete_extension},
};
use watt_lex::lexer::Lexer;
use watt_parse::parser::Parser;

/// Imports resolver structure
///
/// Contains current imported files, contains
/// builtin libraries map by HashMap,
/// where key - library name, value - library path
///
pub struct ImportsResolver<'import_key, 'import_path> {
    imported: RefCell<Vec<String>>,
    libraries: HashMap<&'import_key str, &'import_path str>,
    builtins: Vec<String>,
}
/// Import resolver implementation
/// todo: add could not resolve error instead of file not found
#[allow(unused_qualifications)]
impl<'import_key, 'import_path> ImportsResolver<'import_key, 'import_path> {
    /// New import resolver
    pub fn new() -> Self {
        ImportsResolver {
            imported: RefCell::new(vec![]),
            libraries: HashMap::from([
                ("std.io", "./libs/std/std_io.wt"),
                ("std.gc", "./libs/std/std_gc.wt"),
                ("std.errors", "./libs/std/std_errors.wt"),
                ("std.convert", "./libs/std/std_convert.wt"),
                ("std.time", "./libs/std/std_time.wt"),
                ("std.math", "./libs/std/std_math.wt"),
                ("std.random", "./libs/std/std_random.wt"),
                ("std.fs", "./libs/std/std_fs.wt"),
                ("std.system", "./libs/std/std_system.wt"),
                ("std.crypto", "./libs/std/std_crypto.wt"),
                ("std.strings", "./libs/std/std_strings.wt"),
                ("std.json", "./libs/std/std_json.wt"),
                ("std.ffi", "./libs/std/std_ffi.wt"),
                ("std.net", "./libs/std/std_net.wt"),
            ]),
            builtins: vec!["./libs/base.wt".to_string()],
        }
    }

    /// imports base.wt file, that contains
    /// basic natives and types, such as
    /// `List`, `Map`, `Iterators`, `panic`,
    /// etc.
    ///
    pub fn import_builtins(&mut self) -> Vec<Node> {
        let mut nodes = vec![];

        for builtin in &self.builtins {
            if !self.imported.borrow().contains(builtin) {
                let node_option = self.import(None, &Import::new(None, builtin.to_string(), None));
                if let Some(node) = node_option {
                    nodes.push(node);
                }
            }
        }

        nodes
    }

    /// Resolves import
    ///
    /// 1. Checking `import.file`
    /// - is it a library, if library, gets path from `libraries`,
    ///   else represents `import.file` as `file path`
    ///
    /// 2. Reading the file
    /// - from a resolved path
    ///
    /// 3. Lexing, parsing, analyzing, source files
    /// - leaves only `Import`, `Trait`, `Unit`,
    ///   `Type`, `FnDeclaration`, `Native` node types.
    ///
    /// returns: analyzed AST node
    ///
    fn resolve(&self, addr: Option<Address>, import: &Import) -> Node {
        // resolving path
        let file: &str = if self.libraries.contains_key(import.file.as_str()) {
            self.libraries.get(import.file.as_str()).unwrap()
        } else {
            &import.file
        };
        let path = PathBuf::from(file);

        // getting file name by path
        let file_name = path.file_name().and_then(|x| x.to_str()).unwrap();

        // getting full name prefix, by default
        // it's a file name
        let full_name_prefix = import
            .full_name
            .clone()
            .map_or(delete_extension(file_name), |s| s);

        // reading
        let code = match fs::read_file(addr.clone(), &path) {
            Ok(code) => code,
            Err(err) => match err {
                FileReadError::FileNotFound => match addr {
                    // error if address exists, else panic
                    Some(address) => error!(Error::own_text(
                        address,
                        format!("failed to resolve: {}", import.file),
                        "file not found."
                    )),
                    None => panic!("failed to resolve: {}. file not found.", import.file),
                },
                FileReadError::IoError => match addr {
                    // error if address exists, else panic
                    Some(address) => error!(Error::own_text(
                        address,
                        format!("failed to resolve: {}", import.file),
                        "io error."
                    )),
                    None => panic!("failed to resolve: {}. io error.", import.file),
                },
            },
        };

        // lexing
        let tokens = Lexer::new(&code.chars().collect::<Vec<char>>(), &path).lex();

        // ast
        let ast = Parser::new(tokens, &path, &full_name_prefix).parse();

        // analyzed ast
        Analyzer::new().analyze(&ast);

        // getting necessary nodes
        match ast {
            Node::Block { mut body } => {
                let mut new_body: Vec<Node> = vec![];
                for node in body.drain(..) {
                    match node {
                        Node::Native { .. }
                        | Node::FnDeclaration { .. }
                        | Node::Type { .. }
                        | Node::Unit { .. }
                        | Node::Trait { .. }
                        | Node::Import { .. } => {
                            new_body.push(node);
                        }
                        _ => {}
                    }
                }
                Node::Block { body: new_body }
            }
            _ => {
                panic!("parser returned non-block node as result. report to the developer.");
            }
        }
    }

    /// Resolving wrapper
    ///
    /// Checks if import is already in project, if not
    /// resoles it, and then returns
    ///
    pub fn import(&self, addr: Option<Address>, import: &Import) -> Option<Node> {
        // if file is not imported
        if !self.imported.borrow().contains(&import.file) {
            let node = self.resolve(addr, import);
            self.imported.borrow_mut().push(import.file.clone());
            Option::Some(node)
        }
        // else
        else {
            Option::None
        }
    }
}
