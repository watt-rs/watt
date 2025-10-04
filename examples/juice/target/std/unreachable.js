import {$$match, $$equals, $$EqPattern, $$UnwrapPattern} from "../prelude.js"

import {panic} from "../std/rt.js"

export function unreachable() {
    panic("unreachable created.");
    return -1;
}
