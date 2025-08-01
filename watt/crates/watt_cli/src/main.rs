pub(crate) mod cli;

pub fn main() {
    unsafe {
        cli::cli();
    }
}
