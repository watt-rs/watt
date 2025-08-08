<p align="center">
  <h2 align="center">⛽ Oil.</h2>
  <p align="center"><i>A lightweight, expressive fuel-powered language 🛢️.</i>
</p>

🧴 Oil is dynamically-typed programming language for fuel vm, designed to assist developers.

```oil
use std/io as io
io.println("Hello, Oil!")
```

```oil
use std/io as io

type Juice(multiplier) {
  let juice = 0
  fn apply(amount) {
    self.juice += (self.multiplier * amount)
  }
}

let juice = new Juice(3)
juice.apply(10)
io.println(juice.juice)
```
