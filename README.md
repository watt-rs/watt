Crab - multi-paradigm, dynamic-typed, vm-interpreted programming language. 🦀

Simple example is here. 🐚

```crab
import 'crab::io'

type Crab {
    fn say_hello(name) {
        io.println('Hello, ' + name)
    }
}

crab := new Crab()
crab.say_hello()  
```

Crab is written in pure rust, what provides good
performance and experience 👋🌞