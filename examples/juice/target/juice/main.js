import {$$match, $$equals, $$EqPattern, $$UnwrapPattern} from "../prelude.js"

import * as io from "../std/io.js"
import * as conv from "../std/convert.js"
import * as rt from "../std/rt.js"
import * as result from "../std/result.js"

export class $Juice {
    constructor(multiplier, juice) {
        this.$meta = "Juice";
        this.multiplier = multiplier;
        this.juice = 0;
    }
    apply(amount) {
        let self = this;
        self.juice = self.juice + self.multiplier * amount;
    }
}
export function Juice(multiplier, juice) {
    return new $Juice(multiplier, juice);
}

export function main() {
    let juice = Juice(3);
    juice.apply(result.unwrap(conv.int(io.readln())));
    io.println(juice.juice);
}
