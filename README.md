⚡ **Watt is friednly, lightweight, programming language written in Rust for the JavaScript platform.**

🔦 Simple example:
```
use std/io as io

enum Power {
	On,
	Off
}

type Flashlight(powered: Power) {
  let is_powered = powered

  pub fn power(on: Power) {
    self.is_powered = on
    io.println("power: " <> match self.is_powered {
      Power.On -> "on"
      Power.Off -> "off"
    })
  }
}

fn main() {
  let flashlight = Flashlight(Power.Off())
  flashlight.power(Power.On())
  flashlight.power(Power.Off())
  flashlight.power(Power.On())
}
```
