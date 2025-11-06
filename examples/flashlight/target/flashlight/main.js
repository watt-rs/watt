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

import * as io from "../std/io.js"

export const Power = {
    On: () => ({
        $meta: "Enum",
        $enum: "Power",
        $variant: "On",
    }),
    Off: () => ({
        $meta: "Enum",
        $enum: "Power",
        $variant: "Off",
    })
};

export class $Flashlight {
    constructor(powered) {
        this.$meta = "Type";
        this.$type = "Flashlight";
        this.is_powered = powered
    }
    power(on) {
        let self = this;
        self.is_powered = on
        return io.println("power: " + $$match(self.is_powered, [
            new $$VariantPattern(
                "On",
                function() {
                    return "on"
                }
            ),
            new $$VariantPattern(
                "Off",
                function() {
                    return "off"
                }
            )
        ]))
    }
}
export function Flashlight(powered) {
    return new $Flashlight(powered);
}

export function main() {
    let flashlight = Flashlight(Power.Off())
    flashlight.power(Power.On());
    flashlight.power(Power.Off());
    flashlight.power(Power.On());
}
