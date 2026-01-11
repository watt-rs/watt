/// Imports
use crate::typ::cx::InferCx;

/// Trait for pretty-printing values in the context of type inference.
///
/// This trait is used to generate a human-readable string representation
/// of a value, taking into account information from the type inference context (`InferCx`).
///
/// # Example
///
/// ```ignore
/// impl Pretty for Typ {
///     fn pretty(&self, icx: &mut InferCx) -> String {
///         // Generate a string representation of `self` using `icx`
///     }
/// }
/// ```
///
/// The context `icx` allows the implementation to resolve type variables,
/// substitutions, and other inference-related information when producing the output.
pub trait Pretty {
    /// Returns a human-readable string representation of the value,
    /// using the provided inference context.
    ///
    /// # Parameters
    ///
    /// * `icx` - A mutable reference to the inference context (`InferCx`),
    ///   which may be used to resolve type information or other contextual details.
    ///
    /// # Returns
    ///
    /// A `String` representing the pretty-printed value.
    fn pretty(&self, icx: &mut InferCx) -> String;
}
