// импорт
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Import {
    name: String,
    full_name: Option<String>,
}
// имплементация
impl Import {
    pub fn new(name: String, full_name: Option<String>) -> Self {
        Import {
            name,
            full_name,
        }
    }
}