import {$$match, $$equals} from "../prelude.js"

import * as io from "../std/io.js"
import * as rt from "../std/rt.js"

export function main() {
    let name = io.ask("Enter your name:");
    io.println(name + " on " + rt.which() + " runtime.");
}
