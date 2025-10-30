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
import * as test_runner from "../suit/run.js"
import {assert_eq} from "../suit/assert.js"
import {List} from "../std/list.js"

export function test_success() {
    assert_eq(2 + 2, 4);
}

export function main() {
    let test = Test("test", test_success)
}
