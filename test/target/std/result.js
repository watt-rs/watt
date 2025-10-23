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
                panic("unwrap on error result.");
                return unreachable()
            }
        )
    ])
}
