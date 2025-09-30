import {$$match, $$equals} from "../prelude.js"

export const Option = {
    Some: (element) => ({
        $meta: "Enum",
        $enum: "Option",
        element: element
    }),
    None: () => ({
        $meta: "Enum",
        $enum: "Option",
    })
};
