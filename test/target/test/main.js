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

export class $Apply {
    constructor(function$) {
        this.$meta = "Type";
        this.$type = "Apply";
        this.function$ = function$
    }
}
export function Apply(function$) {
    return new $Apply(function$);
}

export function test(value) {
    let a = Apply(function (a) {
        return value
    })
    return a
}

export function main() {
    let a = test(3)
}
