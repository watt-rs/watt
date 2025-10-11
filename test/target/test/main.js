import {$$match, $$equals, $$EqPattern, $$UnwrapPattern} from "../prelude.js"

import * as io from "../std/io.js"

export class $A {
    constructor(a) {
        this.$meta = "A";
        this.a = 5;
    }
}
export function A(a) {
    return new $A(a);
}

export function function$() {}

export class $class$ {
    constructor() {
        this.$meta = "class";
    }
}
export function class$() {
    return new $class$();
}

export function test(a) {
    io.println(a.a);
}

export function main() {
    class$();
    test(A());
}
