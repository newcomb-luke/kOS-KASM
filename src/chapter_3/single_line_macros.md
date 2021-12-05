# Single-Line Macros

## Declaration

Single-line macros are defined using the `.define` directive:

```
.define NUM         25
```

This creates a macro called `NUM` that has the value of 25.

Single-line macros are more powerful than this though, and support expressions:

```
.define OTHERNUM    NUM + 5
```

Now `OTHERNUM` would be defined as `NUM` plus 5.

So if NUM were redefined later to be 10:

```
.define NUM         10
```

It would change the value of `OTHERNUM` because these values are found at invocation time, or when the macro is actually run.

A long example is:

```
.define NUM         25
.define OTHERNUM    NUM + 5

push OTHERNUM

.define NUM         10

push OTHERNUM
```

becomes:

```
push 30

push 15
```

Not only is this useful for helpful constants, but they can be used with more than just constants:

```
.define PUSH2      push 2

PUSH2
```

becomes:

```
push 2
```

## Arguments

Single-line macros can take arguments as well:

```
.define a(x) 1 + b(x)
.define b(x) 2 * x

push a(2)
```

becomes:

```
push 5
```

Definitions can have multiple arguments like so:

```
.define f(x, y) x + y * y
```

## Undefinition and Overloading

It is also possible to undefine a defintion using the `.undef` directive:

```
.undef f 2
```

Note that the number of arguuments that the macro takes needs to follow it. Not specifying a value defaults the number of arguments to 0.

Single-line macros can be overloaded, meaning that two macros can have the same name as long as they have different numbers of arguments:

```
.define f(x) x + 2
.define f(x, y) x + y
```

This is valid and will not cause a conflict. Which macro gets expanded depends on the number of arguments. Hence why the `.undef` directive
requires the number of arguments to be specified: so that it knows which one to undefine.
