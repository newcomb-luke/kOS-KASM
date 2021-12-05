# Multi-Line Macros

## Declaration

Multi-line macros are defined using the `.macro` and `.endmacro` directives:

```
.macro MULTI
    push 2
    push 2
    add
.endmacro
```

The above macro can be invoked as such:

```
MULTI
```

## Arguments

Multi-line macros can also have arguments just like definitions. The number of arguments must be specified after the macro name.

These arguments are referenced using their positions, and using the `&` symbol:

```
.macro ADD 2
    push &1
    push &2
    add
.endmacro
```

Invoked as:

```
ADD(2, 4)
```

expands to:

```
push 2
push 4
add
```

## Default Arguments

Multi-Line macros can have default arguments as well, and this is specified by using a range.

The first number is the minimum number of required arguments, and the second number is the maximum number of required arguments.

A valid range would be `0-1`

This is placed after the macro's identifier. But if default arguments are used, the default values must be given, separated by commas,
after the range. The number of default values given must match the maximum number of require arguments, minus those that are required.

```
.macro RET 0-1 1
    RET &1
.endmacro
```

This can be invoked as:

```
RET

; Expands to:

ret 1
```

or as:

```
RET(2)

; Expands to:

ret 2
```

Or with a macro such as defined below:

```
.macro DO_SOMETHING 0-2 24, 30
    push &1
    push &2
    sub
.endmacro
```

If this is invoked with only one argument, it defaults to being the first argument:

```
DO_SOMETHING(2)

; Expands to:

push 2
push 30
sub
```

## Undefinition

Just like single-line macros, multi-macros can be undefined using `.unmacro`. Likewise the number of parameters must be given:

```
.macro NOTHING
    nop
.endmacro

.unmacro NOTHING ; This works

.unmacro DO_SOMETHING 2 ; This works because DO_SOMETHING can take 2 arguments
```

As seen in the above example, argument ranges count as that macro taking any number of arguments in the range when being undefined as well.

## Overloading

Although having a number of different arguments per macro is useful, multi-line macros can also be overloaded:

```
.macro PUSH 1
	push &1
.endmacro

.macro PUSH 2
	push &1
	push &2
.endmacro
```

These macros do not conflict because they both have different numbers of arguments. However if the first definition had an argument range of `1-2` they would conflict.
