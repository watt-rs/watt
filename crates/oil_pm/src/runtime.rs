/// Javascript runtime
#[derive(Debug)]
pub enum JsRuntime {
    /// NodeJs runtime
    Node,
    /// Deno runtime
    Deno,
    /// Bun runtime
    Bun,
    /// Common js
    /// for browsers
    Common,
}

/// Default runtime
pub const DEFAULT: JsRuntime = JsRuntime::Deno;
