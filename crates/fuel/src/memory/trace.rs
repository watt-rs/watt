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
    unsafe fn trace(&self, tracer: &mut Tracer);
}

/// Tracer
pub struct Tracer {
    pub heap: FxHashSet<*mut dyn Trace>,
    marked: FxHashSet<*mut dyn Trace>,
    roots: FxHashSet<*mut dyn Trace>,
    guards: FxHashSet<*mut dyn Trace>,
    settings: TracerSettings,
    is_freezed: bool,
}

/// Tracer implementation
impl Tracer {
    /// Creates new tracer
    pub fn new() -> Self {
        Self {
            heap: FxHashSet::default(),
            marked: FxHashSet::default(),
            roots: FxHashSet::default(),
            guards: FxHashSet::default(),
            settings: TracerSettings::default(),
            is_freezed: false,
        }
    }

    /// Freeze/unfreeze
    pub fn freezed(&mut self, value: bool) {
        self.is_freezed = value
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

    /// Resets marked
    pub fn reset_marked(&mut self) {
        self.marked = FxHashSet::default();
    }

    /// Marks `*mut dyn Trace`
    pub fn mark(&mut self, raw: *mut dyn Trace) {
        // If marked doesn't contains passed raw
        if !self.marked.contains(&raw) {
            // Marked
            self.marked.insert(raw);
            // Trace
            unsafe {
                (*raw).trace(self);
            }
        }
    }

    /// Causes garbage collection
    #[allow(unsafe_op_in_unsafe_fn)]
    pub unsafe fn collect_garbage(&mut self) {
        // phase 0. reset.
        self.reset_marked();
        // phase 1. tracing roots.
        for root in self.roots.clone() {
            self.mark(root);
            (*root).trace(self);
        }
        // phase 1. tracing guards.
        for root in self.guards.clone() {
            self.mark(root);
            (*root).trace(self);
        }
        // phase 2. searching unreachables.
        let mut to_free: Vec<*mut dyn Trace> = Vec::new();
        for value in &self.heap {
            if !self.marked.contains(value) {
                to_free.push(*value);
            }
        }
        // phase 3. sweaping.
        for value in to_free {
            self.free(value);
        }
    }

    /// Guards value from sweaping.
    pub unsafe fn guard(&mut self, ptr: *mut dyn Trace) {
        self.guards.insert(ptr);
    }

    /// Unguards value from sweaping.
    pub unsafe fn unguard(&mut self, ptr: *mut dyn Trace) {
        self.guards.remove(&ptr);
    }

    /// Gc Check. Causes gc, if needed.
    #[allow(unsafe_op_in_unsafe_fn)]
    pub unsafe fn check(&mut self) {
        if self.is_freezed {
            return;
        }
        if self.heap.len() > self.settings.threshold {
            self.collect_garbage();
            self.settings.threshold = self.heap.len() * self.settings.threshold_grow_factor as usize
        }
    }
}

/// Macro to simply mark `Gc<T>`
#[macro_export]
macro_rules! mark {
    ($tracer:expr, $gc:expr) => {
        $tracer.mark($gc.raw())
    };
}

/// Marks value
pub fn mark_value(tracer: &mut Tracer, value: Value) {
    match value {
        Value::String(gc) => mark!(tracer, gc),
        Value::Type(gc) => mark!(tracer, gc),
        Value::Fn(gc) => mark!(tracer, gc),
        Value::Native(gc) => mark!(tracer, gc),
        Value::Instance(gc) => mark!(tracer, gc),
        Value::Unit(gc) => mark!(tracer, gc),
        Value::Trait(gc) => mark!(tracer, gc),
        Value::List(gc) => mark!(tracer, gc),
        Value::Any(gc) => mark!(tracer, gc),
        Value::Module(gc) => mark!(tracer, gc),
        _ => {}
    }
}

/// Marks Vec<Value>
pub fn mark_vector(tracer: &mut Tracer, values: &Vec<Value>) {
    for value in values {
        mark_value(tracer, value.clone());
    }
}

/// Marks FxHashMap<String, Value>
pub fn mark_fx_hashmap(tracer: &mut Tracer, values: &FxHashMap<String, Value>) {
    for (_, value) in values {
        mark_value(tracer, value.clone());
    }
}

/// Guard macro
#[macro_export]
macro_rules! guard {
    ($gc:expr) => {
        unsafe {
            (*crate::memory::TRACER.tracer).guard($gc.raw());
        }
    };
}

/// Unguard macro
#[macro_export]
macro_rules! unguard {
    ($gc:expr) => {
        unsafe {
            (*crate::memory::TRACER.tracer).unguard($gc.raw());
        }
    };
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

/// Gc check macro
#[macro_export]
macro_rules! gc_check {
    () => {
        unsafe {
            (*crate::memory::TRACER.tracer).check();
        }
    };
}

/// Gc freeze macro
#[macro_export]
macro_rules! gc_freeze {
    () => {
        unsafe { (*crate::memory::TRACER.tracer).freezed(true) }
    };
}

/// Gc unfreeze macro
#[macro_export]
macro_rules! gc_unfreeze {
    () => {
        unsafe { (*crate::memory::TRACER.tracer).freezed(false) }
    };
}
