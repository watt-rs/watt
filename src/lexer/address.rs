// адрес
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Address {
    pub line: u64,
    pub column: u16,
    pub file: String,
    pub line_text: String,
}
// имплементация
impl Address {
    pub fn new(line: u64, column: u16,
               file: String, line_text: String) -> Address {
        Address { line, column, file, line_text }
    }
}