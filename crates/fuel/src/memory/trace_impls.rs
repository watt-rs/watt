//// Imports
use crate::{
    memory::trace::{Trace, Tracer, mark_vector},
    values::Value,
};
use std::any::Any;

/// Implementation of `Trace` trait
/// for rust's builtin types

/// Trace impls for rust'y `String`
impl Trace for String {
    /// traces string
    unsafe fn trace(&self, _: *mut dyn Trace, _: &mut Tracer) {}
}

/// Trace impls for rust'y `Vec<Value>`
impl Trace for Vec<Value> {
    /// traces string
    unsafe fn trace(&self, self_ptr: *mut dyn Trace, tracer: &mut Tracer) {
        mark_vector(tracer, self_ptr, self);
    }
}

/// Trace impls for rust'y `*mut dyn Any`
impl Trace for *mut dyn Any {
    /// traces string
    unsafe fn trace(&self, _: *mut dyn Trace, _: &mut Tracer) {}
}
