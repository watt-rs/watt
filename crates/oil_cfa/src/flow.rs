/// Imports
use oil_common::address::Address;

/// Flow
pub enum Flow {
    Return(Address),
    Continue(Address),
    Break(Address),
    Normal(Address),
}

/// Implementation
impl Flow {
    /// Gets locations from the flow
    pub fn get_location(&self) -> Address {
        match self {
            Flow::Return(address) => address.clone(),
            Flow::Continue(address) => address.clone(),
            Flow::Break(address) => address.clone(),
            Flow::Normal(address) => address.clone(),
        }
    }
}
