<p align="center">
  <h2 align="center">⛽ Oil.</h2>
  <p align="center"><i>A lightweight, expressive fuel-powered language 🛢️.</i>
</p>

🧴 Oil is statically-typed, type-safe, compiled programming language, designed to assist developers.

```oil
use std/io as io

fn main() {
  io.println("Hello, Oil!")
}
```

```oil
use std/io as io

type Juice(multiplier: int) {
  let multiplier: int = multiplier
  pub let juice: int = 0

  pub fn apply(amount: int) {
    self.juice += (self.multiplier * amount)
  }
}

fn main() {
  let juice: Juice = Juice(3)
  juice.apply(10)

  io.println(juice.juice)
}
```
