/// Imports.
use rustc_hash::FxHashMap;

use crate::memory::gc::Gc;

/// Tracer graph.
pub struct TracerGraph {
    /// References, represented as:
    /// key: Trace.
    /// value: Traces, key referrers to.
    pub references: FxHashMap<*mut dyn Trace, Vec<*mut dyn Trace>>,
}

/// Tracer graph implementation.
impl TracerGraph {
    /// Creates new tracer graph.
    pub fn new() -> Self {
        Self {
            references: FxHashMap::default(),
        }
    }

    /// Adds edge to graph.
    pub fn add_edge(&mut self, from: *mut dyn Trace, to: *mut dyn Trace) {
        self.references.entry(from).or_default().push(to);
    }
}

/// Tracer.
pub struct Tracer {
    /// All heap allocated objects.
    heap: Vec<*mut dyn Trace>,
    /// Tracer graph.
    graph: TracerGraph,
    /// Visited.
    visited: Vec<*mut dyn Trace>,
}
/// Tracer implementation.
impl Tracer {
    /// Creates new tracer.
    pub fn new() -> Self {
        Self {
            heap: Vec::new(),
            graph: TracerGraph::new(),
            visited: Vec::new(),
        }
    }

    /// Resets tracer.
    pub fn reset(&mut self) {
        self.graph = TracerGraph::new();
        self.visited = Vec::new();
    }
    
    /// Traces Gc<T>
    pub unsafe fn trace<T: Trace>(&mut self, what: &Gc<T>) {
        // Ptr
        let ptr = what.raw() as *mut dyn Trace;
        // If not visited already
        if !self.visited.contains(&ptr) {
            // Visited
            self.visited.push(ptr);
            // Trace
            what.trace(ptr, self);
        }
    }

    /// Traces ptr
    pub fn trace_ptr(&mut self, what: *mut dyn Trace) {
        // If not visited already
        if !self.visited.contains(&what) {
            // Visited
            self.visited.push(what);
            // Tracing
            unsafe {
                (*what).trace(what, self);
            }
        }
    }

    /// Marks edge.
    pub fn mark(&mut self, from: *mut dyn Trace, to: *mut dyn Trace) {
        // Adding edge
        self.graph.add_edge(from, to);
        // Tracing to
        self.trace_ptr(to)
    }
}

/// Trace
pub trait Trace {
    unsafe fn trace(&self, self_ptr: *mut dyn Trace, tracer: &mut Tracer);
}

/// Adds edge to the graph
/// with `self_ptr` as from, and
/// provided `$to:expr` as to.
#[macro_export]
macro_rules! mark {
    ($tracer:expr, $self_ptr:expr, $to:expr) => {
        $tracer.mark($self_ptr, $to.raw())
    };
}
