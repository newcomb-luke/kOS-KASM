# Expressions

Expressions can be used in KASM in place of constants, when it is more convenient to do so.

In KASM, expressions either produce an integer, a double, or a boolean value.
An integer is not allowed to be used in place of a boolean, nor a double. Nor a boolean in place of either.

## Supported Operations

#### Unary

* **-** Negate, flips the sign of a number
* **~** Flips all of the bits in a given number/value
* **!** Not, flips a boolean value. If it was true, now it is false, and vice-versa

#### Mathematical

* **+** Addition
* **-** Subtraction
* **\*** Multiplication
* **/** Division
* **%** Modulus (remainder operator)

#### Comparison

* **==** Equals
* **!=** Does not equal
* **<** Less than
* **<=** Less than or equal
* **>** Greater than
* **>=** Greater than or equal

#### Logical

* **&&** - Logical And
* **||** - Logical Or

## Values

In KASM, values can be provided in a few forms.

For integers, hex, decimal, and binary literals are all supported:

```
push 24
push 0x18
push 0b0001_1000 ; Could also be written as 0b00011000
```

## In Practice

Expressions can be used for any instruction operand that supports the type that is produced when the expression is evaluated

```
push 2 + 2

pushv (5.0 / 2.0) > 2.0
```

Becomes:

```
push 4

pushv true
```

Expressions become more useful when introducing macros in future sections.
