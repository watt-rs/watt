/// Imports
use crate::{
    cx::module::ModuleCx,
    resolve::res::Res,
    typ::{Enum, Typ},
};
use watt_ast::ast::{Case, Pattern};
use watt_common::rc_ptr::RcPtr;

/// Exhaustiveness check of pattern matching
pub struct ExMatchCx<'module_cx, 'pkg, 'cx> {
    cx: &'module_cx mut ModuleCx<'pkg, 'cx>,
    value: Typ,
    cases: Vec<Case>,
}

/// Implementation
impl<'module_cx, 'pkg, 'cx> ExMatchCx<'module_cx, 'pkg, 'cx> {
    /// Checks that all possible values
    /// are covered.
    pub fn check(cx: &'module_cx mut ModuleCx<'pkg, 'cx>, value: Typ, cases: Vec<Case>) -> bool {
        // Match cx
        let mut ex = Self { cx, value, cases };
        // Matching value
        match &ex.value {
            // All prelude type possible values
            // could not be covered.
            Typ::Prelude(_) => false,
            // All custom type values
            // could not be covered,
            // because it's a ref type.
            Typ::Custom(_) => false,
            // All enum variant values
            // could be covered, so
            // checking it
            Typ::Enum(en) => ex.check_with_en(en.clone()),
            // All function values
            // cold not be covered,
            // becuase it's a ref type.
            Typ::Function(_) => false,
            // All dyn value
            // could not be covered,
            // because it's a dynamic type
            // with unknown constraints
            Typ::Dyn => false,
            // Could not cover unit
            // values, becuase...
            // it's nothing =)
            Typ::Unit => false,
        }
    }

    /// Checks that all possible
    /// enum variants are covered
    pub fn check_with_en(&mut self, en: RcPtr<Enum>) -> bool {
        // Matched patterns
        let mut matched_patterns = Vec::new();
        // Matching all cases
        for case in self.cases.drain(..) {
            // Matching pattern
            match case.pattern {
                // Unwrap pattern
                Pattern::Unwrap { en: pattern_en, .. } => {
                    match self.cx.infer_resolution(pattern_en) {
                        Res::Variant(_, pattern_variant) => {
                            matched_patterns.push(pattern_variant);
                        }
                        _ => unreachable!(),
                    }
                }
                Pattern::Value(_) => continue,
                Pattern::Variant(var) => match self.cx.infer_resolution(var) {
                    Res::Variant(_, pattern_variant) => {
                        matched_patterns.push(pattern_variant);
                    }
                    _ => unreachable!(),
                },
                Pattern::Default => return true,
            }
        }
        // Deleting duplicates
        matched_patterns.dedup();
        // Checking all patterns covered
        matched_patterns.len() == en.variants.len()
    }
}
