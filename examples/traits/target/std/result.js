import {
    $$match,
    $$equals,
    $$EqPattern,
    $$UnwrapPattern,
    $$WildcardPattern,
    $$BindPattern,
    $$VariantPattern,
} from "../prelude.js"

import {panic} from "../std/rt.js"
import {unreachable} from "../std/unreachable.js"

export const Result = {
    Ok: (value) => ({
        $meta: "Enum",
        $enum: "Result",
        $variant: "Ok",
        value: value
    }),
    Err: (error) => ({
        $meta: "Enum",
        $enum: "Result",
        $variant: "Err",
        error: error
    })
};

export function unwrap(result) {
    return $$match(result, [
        new $$UnwrapPattern(
            "Ok",
            ["value"],
            function($$fields) {
                let value = $$fields.value;
                return value
            }
        ),
        new $$VariantPattern(
            "Err",
            function() {
                return panic("unwrap on error result.")
            }
        )
    ])
}

export function is_ok(result) {
    return $$match(result, [
        new $$VariantPattern(
            "Ok",
            function() {
                return true
            }
        ),
        new $$VariantPattern(
            "Err",
            function() {
                return false
            }
        )
    ])
}

export function is_err(result) {
    return $$match(result, [
        new $$VariantPattern(
            "Ok",
            function() {
                return false
            }
        ),
        new $$VariantPattern(
            "Err",
            function() {
                return true
            }
        )
    ])
}
