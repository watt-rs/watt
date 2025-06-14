<p align="center">
  <h1 align="center" class="huge-text">⚡🍹 Watt</h1>
  <p align="center"><i>A lightweight, expressive scripting language powered by VoltVM</i></p>
</p>

### About
Watt is a lightweight programming language designed to assist developers. 
Built entirely in pure Rust, it offers great performance and a smooth development experience. 🌾💖

### Contribution
Don't be shy, if you can help! We're glad to see your contributions. 

### Examples

Simple example is here. 🍹

```geko
// importing io
import 'std.io'

// a tractor 🚜
type Tractor(storage) {
    // amount of 🌾
    value := 0
    // fill 🌿
    fun fill(value) {
        if self.value + value > storage {
            self.value = storage
            return null
        }
        self.value += value
    }
    // print 📜
    fun print() {
        io.print('tractor value: ')
        io.println(self.value)
    }
}

tractor := new Tractor(100)
tractor.fill(50)
tractor.print()
tractor.fill(70)
tractor.print()
```