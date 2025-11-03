import {
    $$match,
    $$equals,
    $$todo,
    $$range,
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

export function for_loop_test() {
    let r1 = 0
    for (const i of $$range(0, 10, 0)) {
        r1 = i
    }
    assert_eq(r1, 9);
    let r2 = 0
    for (const i of $$range(0, 10, 1)) {
        r2 = i
    }
    assert_eq(r2, 10);
    let r3 = 0
    for (const i of $$range(10, 0, 0)) {
        r3 = i
    }
    assert_eq(r3, 1);
    let r4 = 0
    for (const i of $$range(10, 0, 1)) {
        r4 = i
    }
    return assert_eq(r4, 0)
}

export function main() {
    let tests = List()
    tests.push(Test(additive_test));
    tests.push(Test(multiple_test));
    tests.push(Test(non_exhaustive_warning));
    tests.push(Test(for_loop_test));
    test_runner.run(tests);
}
