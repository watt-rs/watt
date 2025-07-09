use std::{io::Read, path::PathBuf};

// адрес
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Address {
    pub line: u64,
    pub column: u16,
    pub file: Option<PathBuf>,
}

// имплементация
impl Address {
    pub fn new(line: u64, column: u16,
               file: PathBuf) -> Address {
        Address { line, column, file: Some(file) }
    }

    pub fn unknown() -> Address {
        Address { line: 0, column: 0, file: None }
    }

    pub fn get_line(&self) -> Option<String> {
        let filepath = self.file.as_ref()?;

        let mut file = std::fs::OpenOptions::new().read(true).open(filepath).ok()?;
        let mut string = String::new();

        file.read_to_string(&mut string).ok()?;        
        
        return string.split('\n').nth(self.line as usize - 1).map(String::from);
    }
}