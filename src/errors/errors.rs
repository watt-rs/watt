// импорты
use crate::errors::colors;
use crate::lexer::address::Address;

// ошибка
#[derive(Debug, Clone)]
pub struct Error {
    addr: Address,
    text: String,
    hint: String,
}

// паника
#[macro_export]
macro_rules! error {
    ($err:expr) => {
        $err.panic()
    }
}

// имплементация
impl Error {
    // новая ошибка
    pub fn new(addr: Address, text: String, hint: String) -> Self {
        Error {
            addr,
            text,
            hint,
        }
    }

    // вывод
    pub fn panic(&self) {
        // выводим
        println!(
            "┌─ {color}panic:{reset} {text}",
            color = colors::RedColor,
            reset = colors::ResetColor,
            text = self.text,
        );
        println!("│");
        println!("│ {}:", self.addr.file);
        println!("│ {gray}{line}{reset} {text}",
            line = self.addr.line,
            text = self.addr.line_text,
            gray = colors::WhiteColor,
            reset = colors::ResetColor,
        );
        println!("│ {space:count$}^",
                 space = " ",
                 count = self.addr.column as usize
                     + self.addr.line.to_string().len()
        );
        println!("│");
        println!("│ hint: {hint}", hint = self.hint);
        println!("{}", colors::ResetColor);
        // завершаем процесс
        std::process::exit(1);
    }
}
