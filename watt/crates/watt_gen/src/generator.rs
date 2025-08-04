// imports
use crate::visitor::ModuleVisitor;
use std::{collections::HashMap, path::PathBuf};
use watt_analyze::analyzer::Analyzer;
use watt_ast::{ast::Node, import::Import};
use watt_common::{
    address::Address,
    error,
    errors::Error,
    fs::{self, FileReadError},
};
use watt_lex::lexer::Lexer;
use watt_parse::parser::Parser;
use watt_vm::bytecode::{Chunk, ModuleInfo};

/// Bytecode generator result
pub struct GeneratorResult {
    pub builtins: Chunk,
    pub main: Chunk,
    pub modules: HashMap<usize, ModuleInfo>,
}

/// Bytecode generator
pub struct BytecodeGenerator<'import_key> {
    modules: HashMap<usize, ModuleInfo>,
    resolved: HashMap<PathBuf, usize>,
    libraries: HashMap<&'import_key str, PathBuf>,
    builtins: PathBuf,
}
/// Bytecode generator implementation
impl<'import_key> Default for BytecodeGenerator<'import_key> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'import_key> BytecodeGenerator<'import_key> {
    /// Creates new generator
    pub fn new() -> Self {
        BytecodeGenerator {
            modules: HashMap::new(),
            resolved: HashMap::new(),
            libraries: HashMap::from([
                ("std.io", PathBuf::from("./libs/std/std_io.wt")),
                ("std.errors", PathBuf::from("./libs/std/std_errors.wt")),
                ("std.convert", PathBuf::from("./libs/std/std_convert.wt")),
                ("std.time", PathBuf::from("./libs/std/std_time.wt")),
                ("std.math", PathBuf::from("./libs/std/std_math.wt")),
                ("std.random", PathBuf::from("./libs/std/std_random.wt")),
                ("std.fs", PathBuf::from("./libs/std/std_fs.wt")),
                ("std.system", PathBuf::from("./libs/std/std_system.wt")),
                ("std.crypto", PathBuf::from("./libs/std/std_crypto.wt")),
                ("std.strings", PathBuf::from("./libs/std/std_strings.wt")),
                ("std.json", PathBuf::from("./libs/std/std_json.wt")),
                ("std.ffi", PathBuf::from("./libs/std/std_ffi.wt")),
                ("std.net", PathBuf::from("./libs/std/std_net.wt")),
            ]),
            builtins: PathBuf::from("./libs/base.wt"),
        }
    }

    /// Compiles module
    pub fn compile_module(
        &mut self,
        addr: Option<Address>,
        import: String,
        path: PathBuf,
    ) -> ModuleInfo {
        // reading
        let code = match fs::read_file(addr.clone(), &path) {
            Ok(code) => code,
            Err(err) => match err {
                FileReadError::FileNotFound => match addr {
                    // error if address exists, else panic
                    Some(address) => error!(Error::own_text(
                        address,
                        format!("failed to resolve: {import}"),
                        "file not found."
                    )),
                    None => panic!("failed to resolve: {import}. file not found."),
                },
                FileReadError::IoError => match addr {
                    // error if address exists, else panic
                    Some(address) => error!(Error::own_text(
                        address,
                        format!("failed to resolve: {import}"),
                        "io error."
                    )),
                    None => panic!("failed to resolve: {import}. io error."),
                },
            },
        };

        // lexing
        let tokens = Lexer::new(&code.chars().collect::<Vec<char>>(), &path).lex();

        // ast
        let ast = Parser::new(tokens, &path).parse();

        // analyzed ast
        Analyzer::new().analyze(&ast);

        // getting necessary nodes
        let final_ast = match ast {
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
        };

        // compiling ast to bytecode
        let mut visitor = ModuleVisitor::new(self);
        let bytecode = visitor.generate(&final_ast);

        // returning bytecode
        ModuleInfo::new(path, bytecode)
    }

    /// Gets path of import
    pub fn path_of(&self, import: &Import) -> PathBuf {
        if self.libraries.contains_key(import.file.as_str()) {
            self.libraries.get(import.file.as_str()).unwrap().clone()
        } else {
            PathBuf::from(import.file.clone())
        }
    }

    /// Creates module if not exists.
    /// If module already exists,
    /// returns it's identifier
    pub fn module(&mut self, import: &Import) -> usize {
        // path
        let path = self.path_of(import);
        // checking already imported
        match self.resolved.get(&path) {
            Some(id) => *id,
            None => {
                let module = self.compile_module(
                    Some(import.addr.clone()),
                    import.file.clone(),
                    path.clone(),
                );
                self.modules.insert(self.modules.len(), module);
                self.resolved.insert(path, self.modules.len() - 1);
                self.modules.len() - 1
            }
        }
    }

    /// Generate
    pub fn generate(&mut self, ast: Node) -> GeneratorResult {
        // builtins module
        let builtins_module = {
            // reading
            let code = match fs::read_file(None, &self.builtins) {
                Ok(code) => code,
                Err(err) => match err {
                    FileReadError::FileNotFound => {
                        panic!("failed to resolve: {:?}. file not found.", &self.builtins)
                    }
                    FileReadError::IoError => {
                        panic!("failed to resolve: {:?}. file not found.", &self.builtins)
                    }
                },
            };

            // lexing
            let tokens = Lexer::new(&code.chars().collect::<Vec<char>>(), &self.builtins).lex();

            // ast
            let ast = Parser::new(tokens, &self.builtins).parse();

            // analyzed ast
            Analyzer::new().analyze(&ast);

            // getting necessary nodes
            let final_ast = match ast {
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
            };

            // compiling ast to bytecode
            let mut visitor = ModuleVisitor::new(self);
            

            // returning bytecode
            visitor.generate(&final_ast)
        };
        // main module
        let main_module = {
            // analyzed ast
            Analyzer::new().analyze(&ast);
            // compiling ast to bytecode
            let mut visitor = ModuleVisitor::new(self);
            
            // returning bytecode
            visitor.generate(&ast)
        };
        // result
        GeneratorResult {
            builtins: builtins_module,
            main: main_module,
            modules: self.modules.drain().collect(),
        }
    }
}
