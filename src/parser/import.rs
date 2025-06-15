use crate::lexer::address::Address;

// импорт
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Import {
    pub addr: Option<Address>,
    pub name: String,
    pub full_name: Option<String>,
}
// имплементация
impl Import {
    pub fn new(addr: Option<Address>, name: String, full_name: Option<String>) -> Self {
        Import {
            addr,
            name,
            full_name,
        }
    }
}