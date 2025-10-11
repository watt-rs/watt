import {$$match, $$equals, $$EqPattern, $$UnwrapPattern} from "../prelude.js"

import {panic} from "../std/rt.js"
import {unreachable} from "../std/unreachable.js"

export const Result = {
    Ok: (value) => ({
        $meta: "Enum",
        $enum: "Result",
        value: value
    }),
    Err: (error) => ({
        $meta: "Enum",
        $enum: "Result",
        error: error
    })
};

export function unwrap(result) {
    let $$match_result = $$match(result, [
        new $$UnwrapPattern(["value"], function($$fields) {
            let value = $$fields.value;
            return value;
        }),
        new $$UnwrapPattern([], function($$fields) {
            panic("unwrap on error result.");
            return unreachable();
        })
    ]);
    if ($$match_result != null && $$match_result != undefined) {
        return $$match_result
    }
}
