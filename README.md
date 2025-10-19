⚡ **Watt is friednly, lightweight, programming language written in Rust for the JavaScript platform.**

🔦 Simple example:
```
import std/io as io

enum Power {
	On,
	Off
}

type Flashlight(powered: Power) {
  let is_powered = powered

  pub fn power(on: Power) {
    self.is_powered = on
    io.println("is powered: " <> is_powered)
  }

	pub fn is_powered(): Power {
	  self.is_powered
	}
}

fn main() {
  let flashlight = Flashlight(Power.Off())
  flashlight.print()
	flashlight.power(Power.On)
	flahslight.print()
}
```
