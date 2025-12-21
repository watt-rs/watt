💡 Watt is a friendly, robust typed, functional programming language written in Rust, designed to bring the convenience of functional programming to the web.

⚡ Features:
1. No null values, no exceptions.
2. Clear error messages. 
3. A practical type system.
4. Simple and easy-to-use pattern matching.

⚠️ Watt is highly WIP!

🦣 ADT:
```
enum Iceberg {
    Large(size: float, mammoth: Mammoth),
    Small(size: float)
}

type Mammoth {
    age: int,
    name: String
}
```

🦖 Logical expressions:
```
type Rex {
    age: int
}

fn is_ancient(m: Rex): bool {
    if m.age > 10000 {
        true
    } else {
        false
    }
}
```

🦜 Loops:
```
use std/io as io

type Parrot {
    name: string
}

fn repeat(p: Parrot) {
    for i in 0..3 {
        io.println(p.name <> " says: hello!");
    }
}
```

🐍 Enums and pattern matching:
```
enum Snake {
    Python(length: int),
    Boa(length: int)
}

fn description(s: Snake): string =
    match s {
        Snake.Python(length) -> "Python, length " <> length
        Snake.Boa(length)    -> "Boa, length " <> length
    }
```

🔦 More loops example:
```
use std/io as io

type Flashlight {
    power: int
}

fn shine(f: Flashlight) {
    loop {
        if f.power == 0 {
            io.println("The light went out");
            return;
        }

        io.println("Light power: " <> f.power);
        f.power = f.power - 1;
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
    for i in 0..10 {
        if i * u.speed > 25 {
            io.println(u.name <> " is winning!");
        }
    }
}
```

🦕 Constants:
```
const MAX_SPEED: float = 50

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

🧭 Panic & todo.
```
type Explorer { name: string, fossils: int }

fn explore(e: Explorer) {
    if e.fossils == 0 { panic "No fossils!" }
    elif e.fossils < 0 { panic }
    elif e.fossils < 5 { todo as "Find more fossils" }
    else { io.println(e.name <> " found many fossils!") }
}

fn main() {
    explore(Explorer("Bob", 0))
    explore(Explorer("Alice", 3))
    explore(Explorer("Carol", 7))
}
```
