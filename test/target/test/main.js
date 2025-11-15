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

export function main() {
    let a = Option.None()
}
