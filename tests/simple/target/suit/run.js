import {
    $$match,
    $$equals,
    $$EqPattern,
    $$UnwrapPattern,
    $$WildcardPattern,
    $$BindPattern,
    $$VariantPattern,
} from "../prelude.js"

import {Test} from "../suit/test.js"
import {List} from "../std/list.js"
import {Option} from "../std/option.js"
import * as option from "../std/option.js"
import * as io from "../std/io.js"
import * as conv from "../std/convert.js"

export function run(tests) {
    let successfull = 0
    let fault = 0
    let current = 0
    while (current < tests.len()) {
        let test = option.unwrap(tests.get$(current))
        $$match(test.run(), [
            new $$UnwrapPattern(
                "Some",
                ["value"],
                function($$fields) {
                    let value = $$fields.value;
                    fault = fault + 1
                    return io.println("× error on " + test.name + ": " + conv.string(value))
                }
            ),
            new $$VariantPattern(
                "None",
                function() {
                    successfull = successfull + 1
                }
            )
        ]);
        current = current + 1
    }
    return io.println("✓: " + conv.string(successfull) + ", ×: " + conv.string(fault))
}
