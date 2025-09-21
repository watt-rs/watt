import {Option} from "../../std/option"

class $Flower {
    constructor(water_level) {
        this.$meta = "Flower";
        this.water_level = water_level;
    }
    water(add) {
        self.water_level = self.water_level + add;
    }
}
function Flower(water_level) {
    return new $Flower(water_level);
}

const Pot = {
    Full: (flower) => ({
        $meta: "Enum",
        $enum: "Pot",
        flower: flower
    }),
    Nothing: () => ({
        $meta: "Enum",
        $enum: "Pot",
    })
};

function main() {
    let pot = Option.Some(Pot.Full(Flower(15)));
    $match(pot, [
        new UnwrapPattern(["element"], function(fields) {
            let element = fields.element;
            let int = element;
            if ($equals(int, 14)) {
                let a = int;
            }
            else if ($equals(int, 15)) {
                let b = int;
            }
            else if (true) {
                let c = int;
            }
        }),
        new EqPattern(Option.None(), function() {})
    ])
}
