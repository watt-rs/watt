// значение
#[derive(Clone, Debug)]
pub enum Value {
    Float(f64),
    Int(i64),
    String(String),
    Bool(bool),
    Null
}