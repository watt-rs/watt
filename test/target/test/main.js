import {
    $$match,
    $$equals,
    $$EqPattern,
    $$UnwrapPattern,
    $$WildcardPattern,
    $$BindPattern,
    $$VariantPattern,
} from "../prelude.js"

import * as io from "../std/io.js"

export function main() {
    let a = (() => {
        if (true) {
            return true
        }
        else {
            return false
        }
    })();
    io.println(a);
}
