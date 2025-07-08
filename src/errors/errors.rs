// импорты
use crate::errors::colors;
use crate::lexer::address::Address;
use std::borrow::Cow;

// ошибка
#[derive(Debug, Clone)]
pub struct Error {
    addr: Address,
    text: Cow<'static, str>,
    hint: Cow<'static, str>,
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
    pub fn new(addr: Address, text: &'static str, hint: &'static str) -> Self {
        Error {
            addr,
            text: Cow::Borrowed(text),
            hint: Cow::Borrowed(hint),
        }
    }

    // новая ошибка
    pub fn own(addr: Address, text: String, hint: String) -> Self {
        Error {
            addr,
            text: Cow::Owned(text),
            hint: Cow::Owned(hint),
        }
    }

    // новая ошибка
    pub fn own_text(addr: Address, text: String, hint: &'static str) -> Self {
        Error {
            addr,
            text: Cow::Owned(text),
            hint: Cow::Borrowed(hint),
        }
    }
    
    // новая ошибка
    #[allow(unused)]
    pub fn own_hint(addr: Address, text: &'static str, hint: String) -> Self {
        Error {
            addr,
            text: Cow::Borrowed(text),
            hint: Cow::Owned(hint),
        }
    }
    
    // вывод
    pub fn panic(&self) -> ! {
        let filename = self.addr.file.as_ref().map_or("-", |v| v);
        let text_line = self.addr.line_text.as_ref().map_or("-", |v| v);

        // выводим
        println!(
            "┌─ {color}panic:{reset} {text}",
            color = colors::RedColor,
            reset = colors::ResetColor,
            text = self.text,
        );
        println!("│");
        println!("│ {}:", filename);
        println!("│ {gray}{line}{reset} {text}",
                 line = self.addr.line,
                 text = text_line,
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
