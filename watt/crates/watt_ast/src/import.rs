// imports
use watt_common::address::Address;

/// Import structure
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Import {
    pub addr: Address,
    pub file: String,
    pub variable: String,
}
/// Import implementation
impl Import {
    /// New import
    ///
    /// * `addr`: optional import address
    /// * `file`: import file
    /// * `variable`: import as
    ///
    pub fn new(addr: Address, file: String, variable: String) -> Self {
        Import {
            addr,
            file,
            variable,
        }
    }
}
