# Sections

In KASM there are two types of sections in a .kasm file:

* Text
* Data

## `.section`

You may change the current section type by writing the `.section` directive:

```
.section .text  ; Changes to a text section
.section .data  ; Changes to a data section
```

## Text Sections

By default, KASM is already in a `.text` section, so you are already familiar with what that means: you can write code in it

#### Example

```
.section .text

.func
_start:
    ...
```

## Data Sections

In KASM if one wanted to declare a global symbol that was not a function, but a value that other KASM files can use without needing
preprocessor includes or copy-and-pasting, we use a `.data` section.

`.data` sections are used to declare symbols and their values

```
.section .data

PI .f64v  3.1415 ; Declares a symbol named PI
                 ; that is a ScalarDouble with a value of 3.1415
```

See [Data Types](../chapter_2/data_types.md) for information on how to declare symbols of all types

