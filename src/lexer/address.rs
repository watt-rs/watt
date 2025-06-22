// адрес
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Address {
    pub line: u64,
    pub column: u16,
    pub file: Option<String>,
    pub line_text: Option<String>,
}
// имплементация
impl Address {
    pub fn new(line: u64, column: u16,
               file: String, line_text: String) -> Address {
        Address { line, column, file: Some(file), line_text: Some(line_text) }
    }

    pub fn unknown() -> Address {
        Address { line: 0, column: 0, file: None, line_text: None }
    }
}