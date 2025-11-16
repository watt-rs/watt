⚡ **Watt is friednly, lightweight, programming language written in Rust for the JavaScript platform.**

🔦 Simple example:
```
enum Power {
	On,
	Off
}

type Flashlight {
  is_powered: Power
}

pub fn power(flashlight: Flashlight, on: Power) {
  flashlight.is_powered = on;
  let result = "power: " <> match flashlight.is_powered {
    Power.On -> "on"
    Power.Off -> "off"
  };
}

fn main() {
  let flashlight = Flashlight(Power.Off());
  power(flashlight, Power.On());
  power(flashlight, Power.Off());
  power(flashlight, Power.On());
}
```
