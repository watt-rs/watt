use crate::vm::bytecode::Chunk;
use crate::vm::table::Table;

// символ
#[derive(Clone, Debug)]
pub struct Symbol {
    pub name: String,
    pub full_name: Option<String>,
}
impl Symbol {
    fn new(name: String, full_name: String) -> Symbol {
        Symbol {name, full_name: Option::Some(full_name)}
    }
    fn by_name(name: String) -> Symbol {
        Symbol {name, full_name:Option::None}
    }
}


// тип
#[derive(Clone, Debug)]
pub struct Type {
    pub name: Symbol,
    pub constructor: Vec<String>,
    pub body: Chunk
}

// экземпляр типа
#[derive(Clone, Debug)]
pub struct Instance {
    pub t: Type,
    pub fields: *mut Table
}

// юнит
#[derive(Clone, Debug)]
pub struct Unit {
    pub name: Symbol,
    pub fields: *mut Table,
    pub body: Chunk
}

// владелец функции
#[derive(Clone, Debug)]
pub enum FnOwner {
    Unit(*mut Unit),
    Instance(*mut Instance),
    Null
}

// функция
#[derive(Clone, Debug)]
pub struct Fn {
    pub name: Symbol,
    pub body: Chunk,
    pub owner: *mut FnOwner
}

// нативная функция
#[derive(Clone, Debug)]
pub struct Native {
    pub name: Symbol,
    pub value: fn(i16,bool) // -> ControlFlow<>
}

// значение
#[derive(Clone, Debug)]
pub enum Value {
    Float(f64),
    Int(i64),
    String(String),
    Bool(bool),
    Type(Type),
    Fn(*mut Fn),
    Native(*mut Native),
    Instance(*mut Instance),
    Unit(*mut Unit),
    Null
}