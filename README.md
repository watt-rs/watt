<p align="center">
  <h2 align="center">⛽ Oil.</h2>
  <p align="center"><i>A lightweight, expressive fuel-powered language 🛢️.</i>
</p>

🧴 Oil is dynamically-typed programming language for fuel vm, designed to assist developers.

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
