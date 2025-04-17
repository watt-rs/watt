use crate::lexer::address::Address;
use crate::colors;
/*
Тип ошибки
 */
pub enum ErrorType {
    Parsing,
    Runtime,
    Semantic,
    Compilation
}

/*
Ошибка
 */
pub struct Error {
    error_type: ErrorType,
    addr: Address,
    text: String,
    hint: String,
}

impl Error {
    // новая ошибка
    pub fn new(error_type: ErrorType, addr: Address, text: String, hint: String) -> Self {
        Error { error_type, addr, text, hint }
    }

    // вывод
    pub fn print(&self) {
        println!("{color}╭ ⚡ {error_type} error.",
            color=colors::RedColor,
            error_type=match self.error_type {
                ErrorType::Parsing => "parsing",
                ErrorType::Compilation => "compilation",
                ErrorType::Runtime => "runtime",
                ErrorType::Semantic => "semantic",
            }
        );
        println!("│ err: {text}", text=self.text);
        println!("│ at: {filename}:{line}", filename=self.addr.file(), line=self.addr.line());
        println!("│ 💡: {hint}", hint=self.hint);
        println!("╰ {color}", color=colors::ResetColor);
    }
}