> [!IMPORTANT]
> ⚠️ Watt is unstable and highly WIP.

💡 **Watt** is an experimental, friendly, robust programming language written in Rust, designed to bring the convenience of functional programming to the web.

__

☄️ Hello, world!
```
use std/io as io

fn main() {
    io.println("Hello, world!");
}
```
__

⚡ Features:
1. No null values, no exceptions.
2. Clear error messages.
3. A practical type system.
__

🦎 Optional mutability:
```
fn some() {
    let x = 1;
    x = 1; // Error

    let mut y = 1;
    y = 2; // Ok
}
```

🦖 Logical expressions:
```
fn is_ancient(age: real): bool {
    if m.age > 10000 {
        true
    } else {
        false
    }
}
```

📐 External js-functions:
```
extern fn clamp(x: int): int = `
    if (x < 0) return 0;
    if (x > 100) return 100;
    return x;
`

fn lantern_power(raw: int): int {
    clamp(raw)
}
```

🦄 Concatenation:
```
type Unicorn {
    name: string,
    speed: int
}

fn race(u: Unicorn) {
    if u.speed > 25 {
        io.println(u.name <> " is winning!");
    }
}
```

🦕 Constants:
```
const MAX_SPEED: int = 50

type Dinosaur {
    name: string,
    speed: int
}

fn check_speed(d: Dinosaur) {
    if d.speed > MAX_SPEED {
        io.println(d.name <> " is a speedster!")
    } else {
        io.println(d.name <> " is cruising at " <> d.speed)
    }
}

fn main() {
    let t = Dinosaur("T-Rex", 60)
    let tr = Dinosaur("Triceratops", 30)

    check_speed(t)
    check_speed(tr)
}
```

📦 Generics:
```
enum Option[T] {
    Some(value: T),
    None
}

type Box[T] {
    value: T
}
```
