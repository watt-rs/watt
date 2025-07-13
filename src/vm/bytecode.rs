// imports
use crate::{lexer::address::Address, vm::values::Value};
use crate::vm::values::TraitFn;

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
        value: OpcodeValue
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
        full_name: Option<String>,
        params: Vec<String>,
        body: Chunk,
        make_closure: bool,
    },
    AnonymousFn {
        addr: Address,
        params: Vec<String>,
        body: Chunk,
        make_closure: bool,
    },    
    DefineType {
        addr: Address,
        name: String,
        full_name: Option<String>,
        constructor: Vec<String>,
        body: Chunk,
        impls: Vec<String>
    },
    DefineUnit {
        addr: Address,
        name: String,
        full_name: Option<String>,
        body: Chunk,
    },
    DefineTrait {
        addr: Address,
        name: String,
        full_name: Option<String>,
        functions: Vec<TraitFn>,
    },
    Define {
        addr: Address,
        name: String,
        value: Chunk,
        has_previous: bool,
    },
    Set {
        addr: Address,
        name: String,
        value: Chunk,
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
        name: String,
        args: Chunk,
        should_push: bool,
    },
    EndLoop {
        addr: Address,
        current_iteration: bool,
    },
    Ret {
        addr: Address,
        value: Chunk,
    },
    Native {
        addr: Address,
        fn_name: String,
    },
    ErrorPropagation {
        addr: Address,
        value: Chunk,
    },
    Impls {
        addr: Address,
        value: Chunk,
        trait_name: String
    },
    DeleteLocal {
        addr: Address,
        name: String,
    }
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
                println!("{}", text);
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
            Opcode::Logic { op, .. } => {
                print_indent(indent, format!("logic {op}").as_str());
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
            Opcode::DefineFn { name, full_name, params, body, .. } => {
                print_indent(indent, format!("fn '{name}' '{full_name:?}'").as_str());
                print_indent(indent + 1, "params:");
                for param in params {
                    print_indent(indent + 2, format!("{param}").as_str());
                }
                print_indent(indent + 1, "body:");
                print_chunk(indent + 2, body);
            }
            Opcode::AnonymousFn { params, body, .. } => {
                print_indent(indent, "anonymous_fn");
                print_indent(indent + 1, "params:");
                for param in params {
                    print_indent(indent + 2, format!("{param}").as_str());
                }
                print_indent(indent + 1, "body:");
                print_chunk(indent + 2, body);
            }
            Opcode::DefineType { name, full_name, constructor, body, .. } => {
                print_indent(indent, format!("define_type '{name}' '{full_name:?}'").as_str());
                print_indent(indent + 1, "constructor:");
                for param in constructor {
                    print_indent(indent + 2, format!("{param}").as_str());
                }
                print_indent(indent + 1, "body:");
                print_chunk(indent + 2, body);                
            }
            Opcode::DefineUnit { name, full_name, body, .. } => {
                print_indent(indent, format!("define_unit '{name}' '{full_name:?}'").as_str());
                print_indent(indent + 1, "body:");
                print_chunk(indent + 2, body);
            }
            Opcode::DefineTrait { name, full_name, functions, .. } => {
                print_indent(indent, format!("define_trait '{name}' '{full_name:?}'").as_str());
                print_indent(indent + 1, "functions:");
                for function in functions {
                    print_indent(indent + 2, format!("{function:?}").as_str());
                }
            }
            Opcode::Define { name, value, has_previous, .. } => {
                print_indent(indent, format!("define '{name}'").as_str());
                print_indent(indent + 1, format!("has_previous:{has_previous}").as_str());
                print_indent(indent + 1, "value:");
                print_chunk(indent + 2, value);
            }
            Opcode::Set { name, value, has_previous, .. } => {
                print_indent(indent, format!("set '{name}'").as_str());
                print_indent(indent + 1, format!("has_previous:{has_previous}").as_str());
                print_indent(indent + 1, "value:");
                print_chunk(indent + 2, value);                
            }
            Opcode::Load { name, has_previous, should_push, .. } => {
                print_indent(indent, format!("load '{name}', should_push:{should_push}").as_str());
                print_indent(indent + 1, format!("has_previous:{has_previous}").as_str());
            }
            Opcode::Call { name, has_previous, should_push, args, .. } => {
                print_indent(indent, format!("call '{name}', should_push:{should_push}").as_str());
                print_indent(indent + 1, format!("has_previous:{has_previous}").as_str());
                print_indent(indent + 1, "args:");
                print_chunk(indent + 2, args);
            }
            Opcode::Duplicate { .. } => {
                print_indent(indent, "duplicate");
            }
            Opcode::Instance { name, args, should_push, .. } => {
                print_indent(indent, format!("instance '{name}', should_push:{should_push}").as_str());
                print_indent(indent + 1, "args:");
                print_chunk(indent + 2, args);
            }
            Opcode::EndLoop { current_iteration, .. } => {
                print_indent(indent, format!("break, current iteration:{current_iteration}").as_str());
            }
            Opcode::Ret { value, .. } => {
                print_indent(indent, "return");
                print_indent(indent + 1, "value:");
                print_chunk(indent + 2, value);
            }
            Opcode::Native { fn_name, .. } => {
                print_indent(indent, format!("native {}", fn_name).as_str());
            }
            Opcode::ErrorPropagation { value, .. } => {
                print_indent(indent, "error_propagation");
                print_indent(indent + 1, "value:");
                print_chunk(indent + 2, value);
            }
            Opcode::Impls { value, trait_name, .. } => {
                print_indent(indent, format!("impls {:?}", trait_name).as_str());
                print_indent(indent + 1, "value:");
                print_chunk(indent + 2, value);
            }
            Opcode::DeleteLocal { name, .. } => {
                print_indent(indent, format!("delete_local {}", name).as_str());
            }
        }
    }
}