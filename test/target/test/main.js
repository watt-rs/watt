import {$$match, $$equals, $$EqPattern, $$UnwrapPattern} from "../prelude.js"

import * as io from "../std/io.js"

export function test() {
    if (3 > 5) {
        return 5;
    }
    else if (true) {
        return 7;
    }
}

export function main() {
    io.println("Hello, world!");
}
