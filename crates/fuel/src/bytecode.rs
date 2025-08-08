use crate::{
    memory::{trace::Trace, trace::Tracer},
    values::{TraitFn, Value},
};
use oil_common::address::Address;
/// Imports
use std::path::PathBuf;

/// Module info
#[derive(Clone, Debug)]
pub struct ModuleInfo {
    pub path: PathBuf,
    pub chunk: Chunk,
}
/// Module info implementation
impl ModuleInfo {
    pub fn new(path: PathBuf, chunk: Chunk) -> Self {
        ModuleInfo { path, chunk }
    }
}

/// Opcodes chunk
#[derive(Clone, Debug)]
pub struct Chunk {
    opcodes: Vec<Opcode>,
}
/// Chunk implementation
impl Chunk {
    /// New chunk
    pub fn new(chunk: Vec<Opcode>) -> Self {
        Chunk { opcodes: chunk }
    }
    /// Creates chunk from opcode
    pub fn of(op: Opcode) -> Self {
        Chunk { opcodes: vec![op] }
    }
    /// Get opcodes list
    pub fn opcodes(&self) -> &Vec<Opcode> {
        &self.opcodes
    }
}
/// Trace implementation for chunk
impl Trace for Chunk {
    unsafe fn trace(&self, _: &mut Tracer) {}
}

/// Opcode value
#[derive(Clone, Debug)]
pub enum OpcodeValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Raw(Value),
}

/// Opcode
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum Opcode {
    Push {
        addr: Address,
        value: OpcodeValue,
    },
    Pop {
        addr: Address,
    },
    Bin {
        addr: Address,
        op: String,
    },
    Neg {
        addr: Address,
    },
    Bang {
        addr: Address,
    },
    Cond {
        addr: Address,
        op: String,
    },
    Logic {
        addr: Address,
        a: Chunk,
        b: Chunk,
        op: String,
    },
    If {
        addr: Address,
        cond: Chunk,
        body: Chunk,
        elif: Option<Chunk>,
    },
    Loop {
        addr: Address,
        body: Chunk,
    },
    DefineFn {
        addr: Address,
        name: String,
        params: Vec<String>,
        body: Chunk,
    },
    AnonymousFn {
        addr: Address,
        params: Vec<String>,
        body: Chunk,
    },
    DefineType {
        addr: Address,
        name: String,
        constructor: Vec<String>,
        body: Chunk,
        impls: Vec<Chunk>,
    },
    DefineUnit {
        addr: Address,
        name: String,
        body: Chunk,
    },
    DefineTrait {
        addr: Address,
        name: String,
        functions: Vec<TraitFn>,
    },
    Define {
        addr: Address,
        name: String,
        has_previous: bool,
    },
    Store {
        addr: Address,
        name: String,
        has_previous: bool,
    },
    Load {
        addr: Address,
        name: String,
        has_previous: bool,
        should_push: bool,
    },
    Call {
        addr: Address,
        name: String,
        args: Chunk,
        has_previous: bool,
        should_push: bool,
    },
    Duplicate {
        addr: Address,
    },
    Instance {
        addr: Address,
        args: Chunk,
        should_push: bool,
    },
    EndLoop {
        addr: Address,
        current_iteration: bool,
    },
    Ret {
        addr: Address,
    },
    Native {
        addr: Address,
        fn_name: String,
    },
    ErrorPropagation {
        addr: Address,
        should_push: bool,
    },
    Impls {
        addr: Address,
    },
    DeleteLocal {
        addr: Address,
        name: String,
    },
    ImportModule {
        addr: Address,
        id: usize,
        variable: String,
    },
}
/// Opcode Implementation
impl Opcode {
    /// Prints opcode
    /// with nested opcodes
    ///
    /// Just like tree 🌴
    pub fn print(&self, indent: usize) {
        /// Print text with indent
        fn print_indent(indent: usize, text: &str) {
            if indent == 0 {
                println!("{text}");
            } else {
                println!("{space}{text}", space = "  ".repeat(indent));
            }
        }
        /// Print chunk with indent
        fn print_chunk(indent: usize, chunk: &Chunk) {
            for op in &chunk.opcodes {
                op.print(indent);
            }
        }
        // print opcode
        match self {
            Opcode::Push { value, .. } => {
                print_indent(indent, format!("push {value:?}").as_str());
            }
            Opcode::Pop { .. } => {
                print_indent(indent, "pop");
            }
            Opcode::Bin { op, .. } => {
                print_indent(indent, format!("bin {op}").as_str());
            }
            Opcode::Neg { .. } => {
                print_indent(indent, "neg");
            }
            Opcode::Bang { .. } => {
                print_indent(indent, "bang");
            }
            Opcode::Cond { op, .. } => {
                print_indent(indent, format!("cond {op}").as_str());
            }
            Opcode::Logic { op, a, b, .. } => {
                print_indent(indent, format!("logic {op}").as_str());
                print_indent(indent + 1, "a:");
                print_chunk(indent + 2, a);
                print_indent(indent + 1, "b:");
                print_chunk(indent + 2, b);
            }
            Opcode::If { cond, body, .. } => {
                print_indent(indent, "if");
                print_indent(indent + 1, "cond:");
                print_chunk(indent + 2, cond);
                print_indent(indent + 1, "body:");
                print_chunk(indent + 2, body);
            }
            Opcode::Loop { body, .. } => {
                print_indent(indent, "loop");
                print_indent(indent + 1, "body:");
                print_chunk(indent + 2, body);
            }
            Opcode::DefineFn {
                name, params, body, ..
            } => {
                print_indent(indent, format!("fn '{name}'").as_str());
                print_indent(indent + 1, "params:");
                for param in params {
                    print_indent(indent + 2, param.to_string().as_str());
                }
                print_indent(indent + 1, "body:");
                print_chunk(indent + 2, body);
            }
            Opcode::AnonymousFn { params, body, .. } => {
                print_indent(indent, "anonymous_fn");
                print_indent(indent + 1, "params:");
                for param in params {
                    print_indent(indent + 2, param.to_string().as_str());
                }
                print_indent(indent + 1, "body:");
                print_chunk(indent + 2, body);
            }
            Opcode::DefineType {
                name,
                constructor,
                body,
                ..
            } => {
                print_indent(indent, format!("define_type '{name}'").as_str());
                print_indent(indent + 1, "constructor:");
                for param in constructor {
                    print_indent(indent + 2, param.to_string().as_str());
                }
                print_indent(indent + 1, "body:");
                print_chunk(indent + 2, body);
            }
            Opcode::DefineUnit { name, body, .. } => {
                print_indent(indent, format!("define_unit '{name}'").as_str());
                print_indent(indent + 1, "body:");
                print_chunk(indent + 2, body);
            }
            Opcode::DefineTrait {
                name, functions, ..
            } => {
                print_indent(indent, format!("define_trait '{name}'").as_str());
                print_indent(indent + 1, "functions:");
                for function in functions {
                    print_indent(indent + 2, format!("{function:?}").as_str());
                }
            }
            Opcode::Define {
                name, has_previous, ..
            } => {
                print_indent(indent, format!("define '{name}'").as_str());
                print_indent(indent + 1, format!("has_previous:{has_previous}").as_str());
            }
            Opcode::Store {
                name, has_previous, ..
            } => {
                print_indent(indent, format!("set '{name}'").as_str());
                print_indent(indent + 1, format!("has_previous:{has_previous}").as_str());
            }
            Opcode::Load {
                name,
                has_previous,
                should_push,
                ..
            } => {
                print_indent(
                    indent,
                    format!("load '{name}', should_push:{should_push}").as_str(),
                );
                print_indent(indent + 1, format!("has_previous:{has_previous}").as_str());
            }
            Opcode::Call {
                name,
                has_previous,
                should_push,
                args,
                ..
            } => {
                print_indent(
                    indent,
                    format!("call '{name}', should_push:{should_push}").as_str(),
                );
                print_indent(indent + 1, format!("has_previous:{has_previous}").as_str());
                print_indent(indent + 1, "args:");
                print_chunk(indent + 2, args);
            }
            Opcode::Duplicate { .. } => {
                print_indent(indent, "duplicate");
            }
            Opcode::Instance {
                args, should_push, ..
            } => {
                print_indent(
                    indent,
                    format!("instance, should_push:{should_push}").as_str(),
                );
                print_indent(indent + 1, "args:");
                print_chunk(indent + 2, args);
            }
            Opcode::EndLoop {
                current_iteration, ..
            } => {
                print_indent(
                    indent,
                    format!("break, current iteration:{current_iteration}").as_str(),
                );
            }
            Opcode::Ret { .. } => {
                print_indent(indent, "return");
            }
            Opcode::Native { fn_name, .. } => {
                print_indent(indent, format!("native {fn_name}").as_str());
            }
            Opcode::ErrorPropagation { .. } => {
                print_indent(indent, "error_propagation");
            }
            Opcode::Impls { .. } => {
                print_indent(indent, "impls");
            }
            Opcode::DeleteLocal { name, .. } => {
                print_indent(indent, format!("delete_local {name}").as_str());
            }
            Opcode::ImportModule { id, variable, .. } => {
                print_indent(indent, format!("import_module {id} as {variable}").as_str());
            }
        }
    }
}
