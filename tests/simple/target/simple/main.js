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

export class $A {
    constructor(b) {
        this.$meta = "Type";
        this.$type = "A";
        this.b = b
    }
}
export function A(b) {
    return new $A(b);
}

export class $B {
    constructor() {
        this.$meta = "Type";
        this.$type = "B";
    }
}
export function B() {
    return new $B();
}

export function main() {
    let a = A(B())
}
