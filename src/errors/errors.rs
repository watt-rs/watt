// imports
use crate::errors::colors;
use crate::lexer::address::Address;
use std::borrow::Cow;

/// Error
#[derive(Debug, Clone)]
pub struct Error {
    addr: Address,
    text: Cow<'static, str>,
    hint: Cow<'static, str>,
}

/// Error macro that panics error
#[macro_export]
macro_rules! error {
    ($err:expr) => {
        $err.panic()
    };
}

/// Error implementation
impl Error {
    /// New error
    pub fn new(addr: Address, text: &'static str, hint: &'static str) -> Self {
        Error {
            addr,
            text: Cow::Borrowed(text),
            hint: Cow::Borrowed(hint),
        }
    }

    /// New error with owned text and own hint
    pub fn own(addr: Address, text: String, hint: String) -> Self {
        Error {
            addr,
            text: Cow::Owned(text),
            hint: Cow::Owned(hint),
        }
    }

    /// New error with owned text and &'static str hint
    pub fn own_text(addr: Address, text: String, hint: &'static str) -> Self {
        Error {
            addr,
            text: Cow::Owned(text),
            hint: Cow::Borrowed(hint),
        }
    }

    /// New error with &'static str text and owned hint
    pub fn own_hint(addr: Address, text: &'static str, hint: String) -> Self {
        Error {
            addr,
            text: Cow::Borrowed(text),
            hint: Cow::Owned(hint),
        }
    }

    /// Panic error
    ///
    /// Prints error information,
    /// address, and then
    /// ends process
    ///
    pub fn panic(&self) -> ! {
        // file_name and line_text
        let file_name = self
            .addr
            .file
            .as_ref()
            .and_then(|x| x.file_name())
            .and_then(|x| x.to_str().map(|y| y.to_string()))
            .unwrap_or(String::from("-"));
        let line_text = self.addr.get_line().unwrap_or(String::from("-"));

        // print
        println!(
            "┌─ {color}panic:{reset} {text}",
            color = colors::RedColor,
            reset = colors::ResetColor,
            text = self.text,
        );
        println!("│");
        println!("│ {file_name}:");
        println!(
            "│ {gray}{line}{reset} {text}",
            line = self.addr.line,
            text = line_text,
            gray = colors::WhiteColor,
            reset = colors::ResetColor,
        );
        println!(
            "│ {space:count$}^",
            space = " ",
            count = self.addr.column as usize + self.addr.line.to_string().len()
        );
        println!("│");
        println!("│ hint: {hint}", hint = self.hint);
        println!("{}", colors::ResetColor);

        // exit process
        std::process::exit(1);
    }
}
