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

export function to_string(value) {
    return value.toString();
}

export function println(value) {
    console.log(value);
}

export class $Basket {
    constructor(item) {
        this.$meta = "Type";
        this.$type = "Basket";
        this.item = item
    }
}
export function Basket(item) {
    return new $Basket(item);
}

export const Mushroom = {
    FlyAgaric: (size) => ({
        $meta: "Enum",
        $enum: "Mushroom",
        $variant: "FlyAgaric",
        size: size
    }),
    Porcini: (weight) => ({
        $meta: "Enum",
        $enum: "Mushroom",
        $variant: "Porcini",
        weight: weight
    }),
    Chanterelle: () => ({
        $meta: "Enum",
        $enum: "Mushroom",
        $variant: "Chanterelle",
    })
};

export class $Gatherer {
    constructor(name, basket) {
        this.$meta = "Type";
        this.$type = "Gatherer";
        this.name = name
        this.basket = basket
    }
}
export function Gatherer(name, basket) {
    return new $Gatherer(name, basket);
}

export function pick(g, mush) {
    g.basket = Basket(mush)
    let msg = $$match(g.basket.item, [
        new $$UnwrapPattern(
            "FlyAgaric",
            ["size"],
            function($$fields) {
                let size = $$fields.size;
                return "found a poisonous fly agaric of size " + to_string(size)
            }
        ),
        new $$UnwrapPattern(
            "Porcini",
            ["weight"],
            function($$fields) {
                let weight = $$fields.weight;
                return "found a porcini mushroom weighing " + to_string(weight)
            }
        ),
        new $$VariantPattern(
            "Chanterelle",
            function() {
                return "found a chanterelle"
            }
        )
    ])
    println(g.name + ": " + msg);
}

export function main() {
    let g = Gatherer("LittleForester", Basket(Mushroom.Chanterelle()))
    pick(g, Mushroom.Porcini(120));
    pick(g, Mushroom.FlyAgaric(42));
    pick(g, Mushroom.Chanterelle());
}
