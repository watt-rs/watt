# Control Flow 🔌

In this chapter, we will talk about the control flow in Watt. We will learn:
- if, elif, else statements for conditional logic.
- loops, such as while, for with break and continue.
- functions and value return.

## Table of contents 📚
- [Conditional Logic 🧶](#conditional-logic-🧶)
- [Loops ♾️](#loops-♾️)
- [Types 📐](#types-📐)

## Conditional Logic 📦

Let's learn if, elif, else statements.

The 'if* * statement allows you to branch your code depending on logical expressions.
You provide a logical expression and then a body, that will be executed if logical expression returns true.

*main.wt:*
```watt
import 'std.io'

number := 3
if number < 5 {
    io.println('number < 5!')
} 
```
Output:
```
number < 5!
```

This code prints *number < 5* if *number* variable value is less than 5.
But what if we want to handle other cases, like *number > 5*. We need *else* and *elif* op-s.

*main.wt:*
```watt
import 'std.io'

number := 11
if number < 5 {
    io.println('number < 5!')
} 
elif number > 5 {
    io.println('number > 5!')
}
elif number > 10 {
    io.println('number > 10!')
]
else {
    io.println('number is ' + number)
}
```
Output:
```
number > 10!
```

*elif* op means do something if previous *if* statement didn't execute, if *some condition*.
*else* op means do something if no one *if* statement didn't execute.

Now let's research all possible conditional op-s.

| Op   |                                                            Description |
|:-----|-----------------------------------------------------------------------:|
| `==` |                                          checking two values are equal |
| `!=` |                                      checking two values are not equal |
| `>`  |                             checking left values is greater than right |
| `>=` | checking left values is greater than right value or equals right value |
| `<`  |                                checking left values is less than right |
| `<=` |     checking left values is less than right value or equals left value |

We have also some logical op-s. Will explain it with simple code example:

*main.wt*:
```watt
import 'std.io'

a := 3
b := 4

if a > 3 and b > 4 {
    io.println('a > 3 and b > 4')
}
elif a < 3 or b < 4 {
    io.println('a < 3 or b < 4')
}
else {
    io.println('a == 3 and b == 3')
}
```

*and* returns true, if left and right conditional expressions returns true
For example:
```watt
3 > 5 and 4 < 5 // false
3 < 5 and 4 < 5 // true
```

*or* returns true, if left, right or both conditional expression returns true
For example:
```watt
3 > 5 or 4 < 5 // true
3 < 5 or 4 < 5 // true
3 > 5 or 4 > 5 // false
```

We also have *unary bang op*, what it means:
```watt
if !(3 > 4) {
    io.println('3 < 4')
}
```
Output:
```
3 < 4
```

This code is equivalent to:
```watt
if 3 < 4 {
    io.println('3 < 4')
}
```

So, we can call bang op a *logical reverse* op, this op reverses true to false, false to true.