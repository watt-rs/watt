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

import {sum} from "../fluorine/main.js"

export function println(text) {
    console.log(text);
}

export function main() {
    return println(sum(3, 4))
}
