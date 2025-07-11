// imports
use crate::executor::executor;
use crate::parser::import::Import;
use core::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use crate::lexer::address::Address;
use crate::parser::ast::Node;

/// Imports resolver structure
/// 
/// Contains current imported files, contains 
/// builtin libraries map by HashMap, 
/// where key - library name, value - library path
/// 
pub struct ImportsResolver<'import_key, 'import_path> {
    imported: RefCell<Vec<String>>,
    libraries: HashMap<&'import_key str, &'import_path str>,
    builtins: Vec<String>
}
/// Import resolver implementation
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
                ("std.typeof", "./libs/std/std_typeof.wt"),
                ("std.time", "./libs/std/std_time.wt"),
                ("std.math", "./libs/std/std_math.wt"),
                ("std.random", "./libs/std/std_random.wt"),
                ("std.fs", "./libs/std/std_fs.wt"),
                ("std.system", "./libs/std/std_system.wt"),
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
            if !self.imported.borrow().contains(&builtin) {
                let node_option = self.import(
                    None,
                    &Import::new(None, builtin.to_string(), None)
                );
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
    /// else represents `import.file` as `file path`
    /// 
    /// 2. Reading the file
    /// - from a resolved path
    /// 
    /// 3. Lexing, parsing, analyzing, source files
    /// - leaves only `Import`, `Trait`, `Unit`,
    /// `Type`, `FnDeclaration`, `Native` node types.  
    /// 
    /// returns: analyzed AST node
    /// 
    fn resolve(
        &self,
        addr: Option<Address>,
        import: &Import
    ) -> Node {
        // file
        let file: &str = if self.libraries.contains_key(import.file.as_str()) {
            self.libraries.get(import.file.as_str()).unwrap()
        } else {
            &import.file
        };
        let path = PathBuf::from(file);
        let code = executor::read_file(addr, &path);
        
        // lexing
        let tokens = executor::lex(
            &path,
            &code.chars().collect::<Vec<char>>(),
            false,
            false
        );
        
        // parsing
        let ast = executor::parse(
            &path,
            tokens.unwrap(),
            false,
            false,
            &import.full_name
        );
        
        // analyzing
        let mut analyzed = executor::analyze(
            ast.unwrap()
        );
        
        // getting necessary nodes
        let result: Node;
        if let Node::Block { body } = &mut analyzed {
            let mut new_body: Vec<Node> = vec![];
            for node in body.drain(..) {
                match node {
                    Node::Native { .. } |
                    Node::FnDeclaration { .. } |
                    Node::Type { .. } |
                    Node::Unit { .. } |
                    Node::Trait { .. } | 
                    Node::Import { .. } => {
                        new_body.push(node);
                    }
                    _ => {}
                }
            }
            result = Node::Block { body: new_body };
        }
        else {
            panic!("parser returned non-block node as result. report to the developer.");
        }
        
        
        result
    }

    
    /// Resolving wrapper
    /// 
    /// Checks if import is already in project, if not
    /// resoles it, and then returns
    /// 
    pub fn import(
        &self,
        addr: Option<Address>,
        import: &Import
    ) -> Option<Node> {
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
