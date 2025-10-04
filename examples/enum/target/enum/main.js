import {$$match, $$equals, $$EqPattern, $$UnwrapPattern} from "../prelude.js"

import * as io from "../std/io.js"

export class $Flower {
    constructor(water_level) {
        this.$meta = "Flower";
        this.water_level = water_level;
    }
    water(amount) {
        let self = this;
        self.water_level = self.water_level + amount;
    }
}
export function Flower(water_level) {
    return new $Flower(water_level);
}

export const Pot = {
    Full: (flower) => ({
        $meta: "Enum",
        $enum: "Pot",
        flower: flower
    }),
    Empty: () => ({
        $meta: "Enum",
        $enum: "Pot",
    })
};

export function main() {
    let pot = Pot.Full(Flower(15));
    let $$match_result = $$match(pot, [
        new $$UnwrapPattern(["flower"], function($$fields) {
            let flower = $$fields.flower;
            io.println(flower);
        }),
        new $$EqPattern(Pot.Empty(), function() {
            io.println("empty");
        })
    ]);
    if ($$match_result != null && $$match_result != undefined) {
        return $$match_result
    }
}
