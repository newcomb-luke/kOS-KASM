# Functions

In KASM, just like KerboScript, you can create functions that can be called later in your programs, as a way to reuse code.

## Declaration

You already know how to define a function, our `_start` function:

```
.func
_start:
    ...
```

You can use the `.func` directive to define other functions as well:

```
.func
add_two:
    ...
```

You write code in the same manner when you are defining a user-defined function as in the `_start` function:

```
.func
add_two:
    add
    ...
```

When this function is called, this expects two numbers pushed to the stack to be added.

If you are familiar with C-style functions, or even KerboScript functions, there is the `return` keyword that is
used when you want to go back to where the function is called.

This is done in KASM using the `ret` instruction. `ret` is tied into the kOS variable scope system and takes one
operand: the number of scopes to "pop" when returning. Any values that you want to return are first pushed to the stack.

For our function, this would be:

```
.func
add_two:
    add
    ret 0
```

The sum of the two numbers is already on the stack, so there is no need to push it again. In our case we didn't create any variable
scopes, so we do `ret 0`. If we had created variable scopes, like in the following example:

```
.func
add_two:
    bscp 3, 0 ; Create a new scope with ID 3
    sto "$x"  ; Store the first number in `x`
    sto "$y"  ; Store the second number in `y`
    push "$x" ; Get the value back out of x
    push "$y" ; Same for y
    add
    escp 1
    ret 0
```

Here we create a variable scope, then we end it with `escp 1`, which removes the 1 scope that we have created.

The two lines:

```
escp 1
ret 0
```

Could simply be replaced with:

```
ret 1
```

## Calling a Function

In KASM we can call a function that we have declared by simply writing the name of the function as the first operand
of the `call` instruction:

```
.global _start

.func
_start:
    push @
    push 2
    push 3
    call add_two, #
    call #, "print()"
    pop
    eop

.func
add_two:
    add
    ret 0
```

This would push `2`, and `3`, and then call `add_two`, then print out the result. In this case the result is of course:

```
5
```

This is how you call functions within the same file as the function is declared. In the next section about creating static libraries,
we will go over how to make functions callable from a different file, and how to call them.
