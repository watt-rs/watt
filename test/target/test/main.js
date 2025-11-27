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

export function a(a) {
    return a
}

export function b(v) {
    let annotated = a
    return annotated(v)
}

export function main() {}
