import {
    $$match,
    $$equals,
    $$EqPattern,
    $$UnwrapPattern,
    $$WildcardPattern,
    $$BindPattern,
    $$VariantPattern,
} from "../prelude.js"

export const Option = {
    Some: (element) => ({
        $meta: "Enum",
        $enum: "Option",
        $variant: "Some",
        element: element
    }),
    None: () => ({
        $meta: "Enum",
        $enum: "Option",
        $variant: "None",
    })
};
