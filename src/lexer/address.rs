use std::{io::{BufRead, Read}, path::PathBuf};

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

        let file = std::fs::OpenOptions::new().read(true).open(filepath).ok()?;
        let reader = std::io::BufReader::new(file);

        reader.lines().nth(self.line as usize - 1)?.ok()
    }
}
