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

import {panic} from "../std/rt.js"

export const Option = {
    Some: (value) => ({
        $meta: "Enum",
        $enum: "Option",
        $variant: "Some",
        value: value
    }),
    None: () => ({
        $meta: "Enum",
        $enum: "Option",
        $variant: "None",
    })
};

export function unwrap(option) {
    return $$match(option, [
        new $$UnwrapPattern(
            "Some",
            ["value"],
            function($$fields) {
                let value = $$fields.value;
                return value
            }
        ),
        new $$VariantPattern(
            "None",
            function() {
                return panic("unwrap on `Option.None`")
            }
        )
    ])
}

export function is_some(option) {
    return $$match(option, [
        new $$VariantPattern(
            "Some",
            function() {
                return true
            }
        ),
        new $$VariantPattern(
            "None",
            function() {
                return false
            }
        )
    ])
}

export function is_none(option) {
    return $$match(option, [
        new $$VariantPattern(
            "Some",
            function() {
                return false
            }
        ),
        new $$VariantPattern(
            "None",
            function() {
                return true
            }
        )
    ])
}
