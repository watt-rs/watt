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

### ToDo
- [x] make anonymous founctions expressions
	- [x] make type annotations for functions: ```let a: fn(): int = f```
- [x] make check for main function and module in project
- [ ] implement `for` loop
    - [ ] implement `range` expression 
- [ ] add invalid names as js keywords, such as `class`, `function`, etc...
- [x] foreign functions mechanism
- [x] different runtimes
- [x] add pkg::use_of_app_package_as_dependency error
- [ ] implement `done` keyword
- [ ] allow using expressions in match clauses (in match expressions only)
- [ ] implement `default` case in match
