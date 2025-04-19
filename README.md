Watt - multi-paradigm, dynamic-typed, vm-interpreted programming language. 🦎

Simple example is here. 🍹

```geko
import 'std.io'

type Gecko {
    fn say_hello(name) {
        io.println('Hello, ' + name)
    }
}

gecko := new Gecko()
gecko.say_hello()  
```

Watt is written in pure rust, what provides good
performance and experience 👋🌞