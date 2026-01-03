/// Imports
use crate::{
    cx::module::ModuleCx,
    errors::ExError,
    typ::{
        res::Res,
        typ::{Enum, EnumVariant, PreludeType, Typ},
    },
};
use ecow::EcoString;
use id_arena::Id;
use watt_ast::ast::{Case, Pattern};
use watt_common::{address::Address, bail, skip};

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
                PreludeType::Bool => ex.check_bool_values_covered(),
                _ => ex.has_default_pattern(&ex.cases),
            },
            // All custom type values
            // could not be covered,
            // because it's a ref type.
            //
            // So, checking for default patterns
            // `BindTo` and `Wildcard`
            Typ::Struct(_, _) => ex.has_default_pattern(&ex.cases),
            // All enum variant values
            // could be covered, so
            // checking it
            //
            // So, checking for default patterns
            // `BindTo` and `Wildcard`
            Typ::Enum(en, _) => ex.check_enum_variants_covered(en.clone()),
            // All function values
            // cold not be covered,
            // becuase it's a ref type.
            //
            // So, checking for default patterns
            // `BindTo` and `Wildcard`
            Typ::Function(_, _) => ex.has_default_pattern(&ex.cases),
            // Could not cover unit
            // values, becuase...
            // it's nothing =)
            //
            // So, checking for default patterns
            // `BindTo` and `Wildcard`
            Typ::Unit => ex.has_default_pattern(&ex.cases),
            // All unbounds values
            // could not be covered,
            // because it's a unknown type
            // with unknown constraints
            //
            // So, checking for default patterns
            // `BindTo` and `Wildcard`
            Typ::Unbound(_) => ex.has_default_pattern(&ex.cases),
            // All generic values
            // could not be covered,
            // because it's a unknown type
            // with unknown constraints
            //
            // So, checking for default patterns
            // `BindTo` and `Wildcard`
            Typ::Generic(_) => ex.has_default_pattern(&ex.cases),
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
    fn check_bool_values_covered(&mut self) -> bool {
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
                _ => skip!(),
            };
        }
        // If not not matched
        (true_matched && false_matched) || self.has_default_pattern(&self.cases)
    }

    /// Ensures all enum patterns are consistent
    fn ensure_enum_patterns_consistent(
        &mut self,
        location: Address,
        pat1: &Pattern,
        pat2: &Pattern,
    ) {
        /// Collects all patterns from single.
        /// Given pattern can collect
        /// many patterns if pattern is `Pattern::Or`.
        fn collect_patterns(pattern: &Pattern) -> Vec<Pattern> {
            let mut patterns = Vec::new();
            match pattern {
                Pattern::Or(pat1, pat2) => {
                    patterns.append(&mut collect_patterns(pat1));
                    patterns.append(&mut collect_patterns(pat2));
                }
                pattern => patterns.push(pattern.clone()),
            }
            patterns
        }

        // Collecting all patterns
        let mut collected_patterns = Vec::new();
        collected_patterns.append(&mut collect_patterns(pat1));
        collected_patterns.append(&mut collect_patterns(pat2));

        // Collecting variant patterns
        let variant_patterns: Vec<Pattern> = collected_patterns
            .into_iter()
            .filter(|pattern| matches!(pattern, Pattern::Unwrap { .. } | Pattern::Variant(_)))
            .collect();

        // Collecting unwrap patterns
        let unwrap_patterns: Vec<Vec<EcoString>> = variant_patterns
            .iter()
            .filter_map(|pattern| {
                if let Pattern::Unwrap { fields, .. } = pattern {
                    Some(fields.iter().map(|(_, name)| name.clone()).collect())
                } else {
                    None
                }
            })
            .collect();

        // If exists at least one unwrap pattern, checking
        // unwrap fields consistent
        if !unwrap_patterns.is_empty() {
            // If `variant_patterns` and `unwrap_patterns`
            // are missmatched, raising error
            if variant_patterns.len() != unwrap_patterns.len() {
                bail!(ExError::EnumPatternsMissmatch {
                    src: self.cx.module.source.clone(),
                    span: location.span.into()
                })
            }

            // Checking that all unwrap patterns fields are same
            let first = unwrap_patterns.first().unwrap();
            for pat in &unwrap_patterns {
                if pat != first {
                    bail!(ExError::EnumUnwrapFieldsMissmatch {
                        src: self.cx.module.source.clone(),
                        span: location.span.into()
                    })
                }
            }
        }
    }

    /// Collects matched variants
    fn collect_enum_variants(&mut self, address: &Address, pattern: &Pattern) -> Vec<EnumVariant> {
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
                // Collecting variants
                variants.append(&mut self.collect_enum_variants(address, pat1));
                variants.append(&mut self.collect_enum_variants(address, pat2));
                // Ensuring that enum patterns are consistent
                self.ensure_enum_patterns_consistent(address.clone(), pat1, pat2)
            }
            _ => return variants,
        }
        variants
    }

    /// Checks that all possible
    /// enum variants are covered
    fn check_enum_variants_covered(&mut self, en: Id<Enum>) -> bool {
        // Matched variants
        let mut matched_variants = Vec::new();
        // Matching all cases
        for case in std::mem::take(&mut self.cases) {
            matched_variants.append(&mut self.collect_enum_variants(&case.address, &case.pattern));
        }
        // Deleting duplicates
        matched_variants.dedup();
        // Checking all patterns covered
        matched_variants.len() == self.cx.tcx.enum_(en).variants.len()
    }
}
