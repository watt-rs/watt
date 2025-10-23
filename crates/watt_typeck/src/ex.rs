/// Imports
use crate::{
    cx::module::ModuleCx,
    resolve::res::Res,
    typ::{Enum, EnumVariant, PreludeType, Typ},
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
            // could not be covered, except boolean.
            Typ::Prelude(typ) => match typ {
                PreludeType::Bool => ex.check_with_bool(),
                _ => ex.has_default_pattern(&ex.cases),
            },
            // All custom type values
            // could not be covered,
            // because it's a ref type.
            //
            // So, checking for default patterns
            // `BindTo` and `Wildcard`
            Typ::Custom(_) => ex.has_default_pattern(&ex.cases),
            // All enum variant values
            // could be covered, so
            // checking it
            //
            // So, checking for default patterns
            // `BindTo` and `Wildcard`
            Typ::Enum(en) => ex.check_with_en(en.clone()),
            // All function values
            // cold not be covered,
            // becuase it's a ref type.
            //
            // So, checking for default patterns
            // `BindTo` and `Wildcard`
            Typ::Function(_) => ex.has_default_pattern(&ex.cases),
            // All dyn value
            // could not be covered,
            // because it's a dynamic type
            // with unknown constraints
            //
            // So, checking for default patterns
            // `BindTo` and `Wildcard`
            Typ::Dyn => ex.has_default_pattern(&ex.cases),
            // Could not cover unit
            // values, becuase...
            // it's nothing =)
            //
            // So, checking for default patterns
            // `BindTo` and `Wildcard`
            Typ::Unit => ex.has_default_pattern(&ex.cases),
        }
    }

    /// Checks that `BindTo` or `Wildcard` pattern exists in cases vec
    fn has_default_pattern(&self, cases: &Vec<Case>) -> bool {
        // Checking for patterns
        for case in cases {
            match case.pattern {
                Pattern::BindTo(_) => return true,
                Pattern::Wildcard => return true,
                _ => continue,
            }
        }
        // Else
        false
    }

    /// Checks that true or false is matched
    /// (true_matched, false_matched)
    fn check_bool_pattern(pattern: &Pattern) -> (bool, bool) {
        // Matching pattern
        match &pattern {
            // Bool pattern
            Pattern::Bool(val) => match val.as_str() {
                "true" => (true, false),
                "false" => (false, true),
                _ => unreachable!(),
            },
            // Or pattern
            Pattern::Or(pat1, pat2) => {
                let first = Self::check_bool_pattern(pat1);
                let second = Self::check_bool_pattern(pat2);
                (first.0 || second.0, first.1 || second.1)
            }
            // Other
            _ => (false, false),
        }
    }

    /// Checks that all possible
    /// bool values (true, false) are covered
    fn check_with_bool(&mut self) -> bool {
        // True matched
        let mut true_matched = false;
        let mut false_matched = false;
        // Matching all cases
        for case in &self.cases {
            match Self::check_bool_pattern(&case.pattern) {
                (true, true) => return true,
                (true, false) => {
                    true_matched = true;
                }
                (false, true) => {
                    false_matched = true;
                }
                _ => {}
            };
        }
        // If not not matched
        return (true_matched && false_matched) || self.has_default_pattern(&self.cases);
    }

    /// Collects matched variants
    fn collect_enum_variants(&mut self, pattern: &Pattern) -> Vec<EnumVariant> {
        // Matched variants
        let mut variants = Vec::new();
        // Matching pattern
        match pattern {
            Pattern::Unwrap { en, .. } => match self.cx.infer_resolution(en.clone()) {
                Res::Variant(_, pattern_variant) => {
                    variants.push(pattern_variant);
                }
                _ => unreachable!(),
            },
            Pattern::Variant(var) => match self.cx.infer_resolution(var.clone()) {
                Res::Variant(_, pattern_variant) => {
                    variants.push(pattern_variant);
                }
                _ => unreachable!(),
            },
            Pattern::Or(pat1, pat2) => {
                variants.append(&mut self.collect_enum_variants(&pat1));
                variants.append(&mut self.collect_enum_variants(&pat2));
            }
            _ => return variants,
        }
        variants
    }

    /// Checks that all possible
    /// enum variants are covered
    fn check_with_en(&mut self, en: RcPtr<Enum>) -> bool {
        // Matched variants
        let mut matched_variants = Vec::new();
        // Matching all cases
        for case in std::mem::take(&mut self.cases) {
            matched_variants.append(&mut self.collect_enum_variants(&case.pattern));
        }
        // Deleting duplicates
        matched_variants.dedup();
        // Checking all patterns covered
        matched_variants.len() == en.variants.len()
    }
}
