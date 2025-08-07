<p align="center">
  <h3 align="center">⛽ Oil</h3>
  <p align="center"><i>A lightweight, expressive scripting language powered by VoltVM ⚡🍹</i>
</p>

🧴 Oil is dynamically-typed programming language, designed to assist developers.

```oil
use io
io.println("Hello, Oil!")
```

```oil
use io

type Juice(juice) {
  fn apply(amount) {
    self.juice += amount
  }
}

let juice = new Juice(100)
juice.apply(10)
io.println(juice.juice)
```
