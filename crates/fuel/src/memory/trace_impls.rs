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
    unsafe fn trace(&self, _: &mut Tracer) {}
}

/// Trace impls for rust'y `Vec<Value>`
impl Trace for Vec<Value> {
    /// traces string
    unsafe fn trace(&self, tracer: &mut Tracer) {
        mark_vector(tracer, self);
    }
}

/// Trace impls for rust'y `*mut dyn Any`
impl Trace for *mut dyn Any {
    /// traces string
    unsafe fn trace(&self, _: &mut Tracer) {}
}
