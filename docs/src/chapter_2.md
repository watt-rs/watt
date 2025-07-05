# Control Flow 🔌

In this chapter, we will talk about the control flow in Watt. We will learn:
- if, elif, else statements for conditional logic.
- loops, such as while, for with break and continue.
- functions and value return.

## Table of contents 📚
- [Conditional Logic 🧶](#conditional-logic-📦)
- [Variable Declarations 🧃](#variable-declarations-🧃)
- [Types 📐](#types-📐)

## Conditional Logic 📦

First of all, let's create code file, for example *main.wt*
Every Watt file should end with *.wt* extension. if you're using two or more
words in file name, you should use underscore to separate them.
For example: *hello_world.wt*, rather than *helloworld.wt*

```
my_project
  /main.wt
```

Now open the main.rs file and let's write our first program, traditional 'Hello, world!'

*main.wt:*
```watt
import 'std.io'
io.println('Hello, world!')
```

Save the file and go back to your terminal window. Now let's run our program.
To run a file, you should use simple following command:

```shell
$ watt main.wt
```
Output:
```
Hello, world!
```

In the program, as you can see, we import the "std.io" library and use io.println to print text to the terminal and go to the next line.

## Variable declarations 🧃

Let's learn how to declare variables. Go back to your code editor and write following code:
```watt
import 'std.io'
a := 3
b := 7
io.println(a + b)
```

Next, save your code file, back to your terminal, and try to run this program, using following command:

```shell
$ watt main.wt
```
Output:
```
10
```

As you can see, program printed 10, result of adding 3 and 7.
Let's research program anatomy.

```
a := 3
b := 7
```

In these lines of code, we're declaring two variables with int values (3, 7).

```watt
io.println(a + b)
```

Next line of code prints the result of adding *a* and *b* variable values.
I think, you understand how to declare variables, but what, if you want change variable name. Let's try following code:

```watt
a := 5
a := 7
```

We will get this error, because we are trying to redeclare a variable:
```err
┌─ panic: a is already defined.
│
│ test.wt:
│ 107 a := 7
│     ^
│
│ hint: you can rename variable.
```

Right way to change variable is '=' (assign) op.

```watt
import 'std.io'
a := 3
a = 5
b := 7
io.println(a + b)
```
Output:
```
12
```

## Types 📐

Watt is a dynamic-typed programming language, that means, we can change variable type at runtime, for example, let's run following code:

```
import 'std.io'
a := 5
a = 'hello'
io.println()
```
Output:
```
hello
```

We changed *a* type from *i64*, to a *string*. Now, let's learn all Watt variable types. Here's a table:


| Type       |                         Description |
|:-----------|------------------------------------:|
| `f64`      |               64-bit floating point |
| `i64`      |                      64-bit integer |
| `bool`     |            Boolean (`true`/`false`) |
| `null`     |                 Represents no value |
| `string`   |                        Textual data |
| `unit`     |               Static type reference |
| `instance` |                       Type instance |
| `trait`    | Trait (like a rust trait) reference |
| `type`     |                      Type reference |