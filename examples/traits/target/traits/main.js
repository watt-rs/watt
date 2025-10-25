import {
    $$match,
    $$equals,
    $$EqPattern,
    $$UnwrapPattern,
    $$WildcardPattern,
    $$BindPattern,
    $$VariantPattern,
} from "../prelude.js"

import * as io from "../std/io.js"

export let Dog = [
    "bark"
]

export class $Dalmatian {
    constructor() {
        this.$meta = "Type";
        this.$type = "Dalmatian";
    }
    bark() {
        let self = this;
        return io.println("Dalmatian: woof! woof!")
    }
}
export function Dalmatian() {
    return new $Dalmatian();
}

export class $Bulldog {
    constructor() {
        this.$meta = "Type";
        this.$type = "Bulldog";
    }
    bark() {
        let self = this;
        return io.println("Bulldog: arf-arf! woof!")
    }
}
export function Bulldog() {
    return new $Bulldog();
}

export function bark(dog) {
    return dog.bark()
}

export function main() {
    let dog = Bulldog()
    let dog2 = Dalmatian()
    bark(dog);
    bark(dog2);
}
