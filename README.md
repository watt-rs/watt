⚡ **Watt is friednly, lightweight, programming language written in Rust for the JavaScript platform.**

🔦🍄 Simple example:
```
/// Imports
use std/convert for to_string
use std/io as io

/// Generic "Basket" that can hold anything —
/// mushrooms, berries, or found artifacts.
type Basket[T] {
    item: T
}

/// Enumeration of mushroom types
enum Mushroom {
    FlyAgaric(size: int),   /// Poisonous fly agaric
    Porcini(weight: int),   /// Porcini mushroom
    Chanterelle,            /// Chanterelle
}

/// Type representing a mushroom gatherer
type Gatherer {
    name: string,
    basket: Basket[Mushroom]
}

/// Function that replaces the contents of the gatherer's basket
pub fn pick(g: Gatherer, mush: Mushroom) {
    g.basket = Basket(mush);

    let msg = match g.basket.item {
        Mushroom.FlyAgaric(s) -> "found a poisonous fly agaric of size " <> s
        Mushroom.Porcini(w)   -> "found a porcini mushroom weighing " <> w
        Mushroom.Chanterelle  -> "found a chanterelle"
    };

    io.println(g.name <> ": " <> msg);
}

/// Example usage
fn main() {
    let g: Gatherer = Gatherer(
        "LittleForester",
        Basket(Mushroom.Chanterelle())
    );

    pick(g, Mushroom.Porcini(120));
    pick(g, Mushroom.FlyAgaric(42));
    pick(g, Mushroom.Chanterelle());
}
```
