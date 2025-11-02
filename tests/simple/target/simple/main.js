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

export function additive_test() {
    assert_eq(2 + 2, 4);
    assert_eq(63 - 17, 46);
}

export function multiple_test() {
    assert_eq(34 * -2, -68);
    assert_eq(250 * 3, 750);
    assert_eq(10 / -2.5, -4);
    return assert_eq(20 / 2, 10)
}

export function non_exhaustive_warning() {
    let result = $$match(true, [
        new $$EqPattern(true, function() {
            return 1
        })
    ])
    return assert_eq(result, 1)
}

export function main() {
    let tests = List()
    tests.push(Test(additive_test));
    tests.push(Test(multiple_test));
    tests.push(Test(non_exhaustive_warning));
    test_runner.run(tests);
}
