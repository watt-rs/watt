// импорты
use crate::{lexer::address::Address, vm::values::Value};
use crate::vm::values::TraitFn;

// чанк
#[derive(Clone, Debug)]
pub struct Chunk {
    opcodes: Vec<Opcode>,
}
// имплементация
impl Chunk {
    pub fn new(chunk: Vec<Opcode>) -> Self {
        Chunk { opcodes: chunk }
    }
    pub fn of(op: Opcode) -> Self {
        Chunk { opcodes: vec![op] }
    }
    pub fn opcodes(&self) -> &Vec<Opcode> {
        &self.opcodes
    }
}

// значение опкода
#[derive(Clone, Debug)]
pub enum OpcodeValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Raw(Value),
}

// опкод
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
