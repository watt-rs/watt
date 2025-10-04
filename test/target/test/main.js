import {$$match, $$equals, $$EqPattern, $$UnwrapPattern} from "../prelude.js"

import * as rt from "../std/rt.js"
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
    return $$match(result, [
        new $$UnwrapPattern(["value"], function($$fields) {
            let value = $$fields.value;
            return value;
        }),
        new $$UnwrapPattern([], function($$fields) {
            rt.panic("unwrap on error result.");
            return unreachable();
        })
    ]);;
}

export function main() {}
