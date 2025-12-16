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

export const Or = {
    A: () => ({
        $meta: "Enum",
        $enum: "Or",
        $variant: "A",
    }),
    B: () => ({
        $meta: "Enum",
        $enum: "Or",
        $variant: "B",
    })
};

export function a(t) {
    return $$match(t, [
        new $$VariantPattern(
            "A",
            function() {
                return 1
            }
        ),
        new $$VariantPattern(
            "B",
            function() {
                return $$todo(todo)
            }
        )
    ])
}

export function b(t) {
    return $$match(t, [
        new $$VariantPattern(
            "A",
            function() {
                return 1
            }
        ),
        new $$VariantPattern(
            "B",
            function() {
                return $$panic(todo)
            }
        )
    ])
}

export function main() {
    b(Or.B());
}
