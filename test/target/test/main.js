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

export class $Mammoth {
    constructor(value) {
        this.$meta = "Type";
        this.$type = "Mammoth";
        this.value = value
    }
}
export function Mammoth(value) {
    return new $Mammoth(value);
}

export function println(value) {
    console.log(value);
}

export function main() {
    let mammoth = Mammoth(3)
    for (const i of $$range(0, 1000000, 0)) {
        println("Hello, world!");
    }
}
