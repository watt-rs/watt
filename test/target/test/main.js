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
import {List} from "../std/list.js"

export function a() {
    return io.println("hello, world!")
}

export function main() {
    a();
}
