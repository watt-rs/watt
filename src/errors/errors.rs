// импорты
use crate::colors;
use crate::lexer::address::Address;

// тип ошибки
#[derive(Debug, Clone)]
pub enum ErrorType {
    Parsing,
    Runtime,
    Semantic,
    Compilation,
}

// ошибка
#[derive(Debug, Clone)]
pub struct Error {
    error_type: ErrorType,
    addr: Address,
    text: String,
    hint: String,
}

// имплементация
impl Error {
    // новая ошибка
    pub fn new(error_type: ErrorType, addr: Address, text: String, hint: String) -> Self {
        Error {
            error_type,
            addr,
            text,
            hint,
        }
    }

    // вывод
    pub fn print(&self) {
        println!(
            "{color}╭ ⚡ {error_type} error.",
            color = colors::RedColor,
            error_type = match self.error_type {
                ErrorType::Parsing => "parsing",
                ErrorType::Compilation => "compilation",
                ErrorType::Runtime => "runtime",
                ErrorType::Semantic => "semantic",
            }
        );
        println!("│ err: {text}", text = self.text);
        println!(
            "│ at: {filename}:{line}",
            filename = self.addr.file(),
            line = self.addr.line()
        );
        println!("│ trace: ");
        println!("│ 💡: {hint}", hint = self.hint);
        println!("╰ {color}", color = colors::ResetColor);
    }
}
