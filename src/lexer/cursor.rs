// курсор
pub struct Cursor<'cursor> {
    code: &'cursor [char],
    pub(crate) current: usize,
}

// имплементация
impl<'cursor> Cursor<'cursor> {
    // новый курсор
    pub fn new(code: &'cursor [char]) -> Self {
        Cursor {
            code,
            current: 0,
        }
    }
    
    // текущий символ
    pub fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.char_at(0)
        }
    }

    // следующий символ
    pub fn next(&self) -> char {
        if self.current + 1 >= self.code.len() {
            '\0'
        } else {
            self.char_at(1)
        }
    }

    // в конце ли относительно текущей позиции
    pub fn is_at_end(&self) -> bool {
        self.current >= self.code.len()
    }

    // в конце ли относительно текущей позиции + оффсет
    pub fn is_at_end_offset(&self, offset: usize) -> bool {
        self.current + offset >= self.code.len()
    }

    // символ с оффсетом
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