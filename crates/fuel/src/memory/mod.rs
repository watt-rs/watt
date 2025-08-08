/// Modules
pub(crate) mod gc;
pub(crate) mod memory;
pub mod trace;
pub(crate) mod trace_impls;

//// Imports
use crate::memory::trace::Tracer;
use std::sync::LazyLock;

/// TracerPtr
pub struct TracerPtr {
    pub tracer: *mut Tracer,
}
/// TracerPtr implementation
impl TracerPtr {
    /// Creates new `TracerPtr`
    pub fn new() -> Self {
        Self {
            tracer: memory::alloc_value(Tracer::new()),
        }
    }
}
/// Tracer
pub static mut TRACER: LazyLock<TracerPtr> = LazyLock::new(|| TracerPtr::new());
