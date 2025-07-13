<!--suppress HtmlDeprecatedAttribute -->
<p align="center">
  <img width="363" height="136" alt="⚡🍹 Watt" src="https://github.com/user-attachments/assets/eb7c78b0-3605-4531-b3a4-d8e8bb164571" />
  <p align="center"><i>A lightweight, expressive scripting language powered by VoltVM ⚡🍹</i>
</p>

### About
Watt is a lightweight programming language designed to assist developers. 
Built entirely in pure Rust, it offers great performance and a smooth development experience. 🌾💖

### Contribution
Don't be shy, if you can help! We're glad to see your contributions. 

### Examples
Simple example is here. 🍹

```zig
// importing io
import 'std.io'

// a tractor 🚜
type Tractor(storage) {
    // amount of 🌾
    value := 0
    // fill 🌿
    fn fill(value) {
        if self.value + value > storage {
            self.value = storage
            return null
        }
        self.value += value
    }
    // print 📜
    fn print() {
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


### ToDo ⌛
- std libraries: math, graphics, http, etc...
- match expr/stmt.
- map expr.
- anonymous fn-s.
