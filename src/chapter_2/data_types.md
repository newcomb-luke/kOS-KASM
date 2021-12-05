# Data Types

In kOS, unlike in many other computing platforms, each value has an associated type.

These types are as follows:

| Argument Type Value      | Type           |
| -----------------------  | :------------: |
| 0	                   | NULL           |
| 1	                   | Bool           |
| 2	                   | Byte           |
| 3	                   | Int16          |
| 4	                   | Int32          |
| 5	                   | Float          |
| 6	                   | Double         |
| 7	                   | String         |
| 8	                   | ARG MARKER     |
| 9	                   | ScalarInt      |
| 10	                   | ScalarDouble   |
| 11	                   | BoolValue      |
| 12	                   | StringValue    |

* **NULL** is a value that is equivalent to "nothing"
* **Bool** is a true or false value
* **Byte** is an integer that can store numbers from -128 to 127
* **Int16** is an integer that can store numbers from -32,768 to 32,767
* **Int32** is an integer that can store numbers from -2,147,483,648 to 2,147,483,647
* **Float** is a number that can have a fractional component such as 2.33
* **Double** is a float, but with double the precision of the fractional component, see [this](https://en.wikipedia.org/wiki/Double-precision_floating-point_format)
* **String** is a "string" of characters, such as "Hello, kOS!"
* **ARG MARKER** is a special value used inside of kOS that marks the end of function "arguments"
* **ScalarInt** is an integer that can store the same numbers as the Int32, but has associated functions inside of kOS
* **ScalarDouble** is a double, but with associated functions inside of kOS
* **BoolValue** is a boolean value, true or false, but with associated functions
* **StringValue** same as a string, but with associated functions

## Inside of KASM

Data types are for the most part handled automatically by KASM

KASM chooses the smallest size data type in order to make the assembled executable file as small as possible.

For example, in the code:

```
push 2
```

That 2 would be stored using the Byte type, meanwhile in:

```
push 200
```

00 would be stored as an `Int16` because it is the smallest type that can hold the value of `200`.

When trying to pass any floating-point number such as `2.33`, KASM will automatically store it as a `Double`.

If you wanted to push a `StringValue` instead of a `String` in KASM, you simply use the pushv pseudoinstruction:

```
push  "Hello"        ; This is a regular String
pushv "kOS!"         ; This will push a StringValue
```

If `pushv` is given an integer, it will be pushed as a `ScalarInt`.

## Pushing Other Types

Simply writing an integer or another value to be the operand to an instruction is simple enough, however there are still two types that are not accounted for:

    NULL
    ARG MARKER

These are shown in KASM code as follows:

```
push # ; This is a NULL
push @ ; This is an ARG MARKER
```

## Declaring Symbols

In KASM there are pieces of data that can be referenced outside of the current file called symbols. They are explained more in the advanced tutorial.

However it is important to note that when declaring symbols in KASM, the type of the symbol must be provided.

An example of how to declare each type of symbol is shown below:
```
.section .data
a  #              ; Null
b  .b    true     ; Bool
c  .i8   120      ; Byte
d  .i16  300      ; Int16
e  .i32  900      ; Int32
f  .f64  2.33     ; Double
g  .s    "Hello"  ; String
h  @              ; Arg Marker
i  .i32v 900      ; ScalarInt
j  .f64v 2.33     ; ScalarDouble
k  .bf   false    ; BoolValue
l  .sv   "kOS"    ; StringValue
```
