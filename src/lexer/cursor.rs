/// Lexer cursor
pub struct Cursor<'cursor> {
    code: &'cursor [char],
    pub(crate) current: usize,
}
/// Cursor implementation
impl<'cursor> Cursor<'cursor> {
    /// New cursor
    pub fn new(code: &'cursor [char]) -> Self {
        Cursor {
            code,
            current: 0,
        }
    }
    
    /// Peeks current char
    pub fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.char_at(0)
        }
    }
    
    /// Peeks next char
    pub fn next(&self) -> char {
        if self.current + 1 >= self.code.len() {
            '\0'
        } else {
            self.char_at(1)
        }
    }
    
    /// Checking `current >= self.code.len()`
    pub fn is_at_end(&self) -> bool {
        self.current >= self.code.len()
    }
    
    /// Gets char at `current + offset`
    pub fn char_at(&self, offset: usize) -> char {
        let index = self.current + offset;
        if self.code.len() > index {
            let c = self.code[index];
            c
        } else {
            '\0'
        }
    }
}