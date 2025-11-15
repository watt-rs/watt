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

export const Result = {
    Ok: (value) => ({
        $meta: "Enum",
        $enum: "Result",
        $variant: "Ok",
        value: value
    }),
    Error: (error) => ({
        $meta: "Enum",
        $enum: "Result",
        $variant: "Error",
        error: error
    })
};

export function main() {
    let a = Result.Ok(3)
    a = Result.Error(4)
    let b = a
}
