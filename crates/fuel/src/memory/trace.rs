//// Imports
use crate::{memory::memory, values::Value};
use rustc_hash::{FxHashMap, FxHashSet};

/// Tracer settings
pub struct TracerSettings {
    pub threshold: usize,
    pub threshold_grow_factor: u8,
    pub debug: bool,
}

/// Tracer settings default
impl Default for TracerSettings {
    fn default() -> TracerSettings {
        TracerSettings {
            threshold: 2048,
            threshold_grow_factor: 2,
            debug: false,
        }
    }
}

/// Trace trait
pub trait Trace {
    unsafe fn trace(&self, self_ptr: *mut dyn Trace, tracer: &mut Tracer);
}

/// Tracer
pub struct Tracer {
    pub heap: FxHashSet<*mut dyn Trace>,
    roots: FxHashSet<*mut dyn Trace>,
    settings: TracerSettings,
}

/// Tracer implementation
impl Tracer {
    /// Creates new tracer
    pub fn new() -> Self {
        Self {
            heap: FxHashSet::default(),
            roots: FxHashSet::default(),
            settings: TracerSettings::default(),
        }
    }

    /// Updates settings
    pub fn settings(&mut self, settings: TracerSettings) {
        self.settings = settings;
    }

    /// Allocates memory in heap,
    /// adds value to tracer heap vector,
    /// returns ptr to allocated value
    pub fn alloc<T: Trace + 'static>(&mut self, value: T) -> *mut T {
        let ptr = memory::alloc_value(value);
        self.heap.insert(ptr);
        ptr
    }

    /// Safely deallocates memory
    /// in heap with drop.
    pub fn free(&mut self, ptr: *mut dyn Trace) {
        self.heap.remove(&ptr);
        memory::free_value(ptr);
    }

    /// Adds root
    pub fn add_root(&mut self, root: *mut dyn Trace) {
        self.roots.insert(root);
    }

    /// Marks `*mut dyn Trace`
    pub fn mark(&mut self, _: *mut dyn Trace, _: *mut dyn Trace) {
        todo!();
    }
}

/// Macro to simply mark `Gc<T>`
#[macro_export]
macro_rules! mark {
    ($tracer:expr, $self_ptr:expr, $gc:expr) => {
        $tracer.mark($self_ptr, $gc.raw())
    };
}

/// Marks value
pub fn mark_value(tracer: &mut Tracer, self_ptr: *mut dyn Trace, value: Value) {
    match value {
        Value::String(gc) => mark!(tracer, self_ptr, gc),
        Value::Type(gc) => mark!(tracer, self_ptr, gc),
        Value::Fn(gc) => mark!(tracer, self_ptr, gc),
        Value::Native(gc) => mark!(tracer, self_ptr, gc),
        Value::Instance(gc) => mark!(tracer, self_ptr, gc),
        Value::Unit(gc) => mark!(tracer, self_ptr, gc),
        Value::Trait(gc) => mark!(tracer, self_ptr, gc),
        Value::List(gc) => mark!(tracer, self_ptr, gc),
        Value::Any(gc) => mark!(tracer, self_ptr, gc),
        Value::Module(gc) => mark!(tracer, self_ptr, gc),
        _ => {}
    }
}

/// Marks Vec<Value>
pub fn mark_vector(tracer: &mut Tracer, self_ptr: *mut dyn Trace, values: &Vec<Value>) {
    for value in values {
        mark_value(tracer, self_ptr, value.clone());
    }
}

/// Marks FxHashMap<String, Value>
pub fn mark_fx_hashmap(
    tracer: &mut Tracer,
    self_ptr: *mut dyn Trace,
    values: &FxHashMap<String, Value>,
) {
    for (_, value) in values {
        mark_value(tracer, self_ptr, value.clone());
    }
}

/// Root macro
#[macro_export]
macro_rules! root {
    ($gc:expr) => {
        unsafe {
            (*crate::memory::TRACER.tracer).add_root($gc.raw());
        }
    };
}

/// Allocates memory
#[macro_export]
macro_rules! alloc {
    ($value:expr) => {
        unsafe { (*crate::memory::TRACER.tracer).alloc($value) }
    };
}
