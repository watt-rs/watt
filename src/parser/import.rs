// imports
use crate::lexer::address::Address;

/// Import structure
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Import {
    pub addr: Option<Address>,
    pub file: String,
    pub full_name: Option<String>,
}
/// Import implementation
impl Import {
    /// New import
    ///
    /// * `addr`: optional import address
    /// * `file`: import file
    /// * `full_name`: optional full name prefix override
    ///
    pub fn new(addr: Option<Address>, file: String, full_name: Option<String>) -> Self {
        Import {
            addr,
            file,
            full_name,
        }
    }
}
