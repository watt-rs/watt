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
import * as conv from "../std/convert.js"

export function assert_eq(a, b) {
    return (() => {
        if (!$$equals(a, b)) {
            return panic("failed on `assert_eq` with: " + conv.string(a) + ", " + conv.string(b))
        }
    })()
}

export function assert_ne(a, b) {
    return (() => {
        if ($$equals(a, b)) {
            return panic("failed on `assert_ne` with: " + conv.string(a) + ", " + conv.string(b))
        }
    })()
}
