use crate::errors::Error;
use crate::vm::values::Value;

#[derive!(Eq, PartialEq, Hash, Debug)]
pub struct Chunk {
    opcodes: Vec<Opcode>
}

impl Chunk {
    pub fn new(chunk: Vec<Opcode>) -> Self {
        Chunk { opcodes: chunk }
    }
}

#[derive!(Eq, PartialEq, Hash, Debug)]
pub enum Opcode {
    Push {
        value: Value
    },
    Pop,
    Bin {
        op: String
    },
    Neg,
    Bang,
    Cond {
        op: String
    },
    Logic {
        op: String
    },
    If {
        cond: Box<Chunk>,
        body: Box<Chunk>,
        elif: Option<Box<Opcode>>,
    },
    Loop {
        body: Box<Chunk>,
    },
    DefineFn {
        name: String,
        full_name: Option<String>,
        params: Vec<String>,
        body: Box<Chunk>,
    },
    DefineType {
        name: String,
        full_name: Option<String>,
        constructor: Vec<String>,
        body: Box<Chunk>,
    },
    DefineUnit {
        name: String,
        full_name: Option<String>,
        body: Box<Chunk>,
    },
    Define {
        name: String,
        value: Box<Chunk>,
        has_previous: bool,
    },
    Set {
        name: String,
        value: Box<Chunk>,
        has_previous: bool,
    },
    Load {
        name: String,
        has_previous: bool,
    },
    Call {
        name: String,
        args: Box<Chunk>,
        has_previous: bool,
    },
    Duplicate,
    Instance {
        name: String,
        args: Box<Chunk>,
    },
    EndLoop {
        current_iteration: bool
    },
    Closure {
        name: String,
    },
    Ret {
        args: Box<Chunk>,
    }
}