/*
Структура адреса
 */
#[derive(Debug)]
#[derive(Clone)]
pub struct Address {
    line: u64,
    file: String,
}

/*
Структура адреса : имплементация
 */
impl Address {
    pub fn new(line: u64, file: String) -> Address {
        Address { line, file }
    }

    pub fn line(&self) -> u64 {
        self.line
    }

    pub fn file(&self) -> String {
        self.file.clone()
    }
}