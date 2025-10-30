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

export function unreachable() {
    return panic("unreachable created.")
}
