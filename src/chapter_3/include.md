# Include

The `.include` directive is a powerful preprocessor directive that allows the programmer to include source code from another file into the current file.

## Example

```
.include "macros.kasm"

PRINT "Hello, world!"
```

This assumes that the source file called `macros.kasm` is in the same directory as the main file, or in the include directory specified by **-i**
and it contains (at least):

```
; macros.kasm

.macro PRINT 1
    push @
    push &1
    call "", "print()"
    pop
.endmacro
```

This causes the source code after the `.include` directive is expanded to effectively be:

```
.macro PRINT 1
    push @
    push &1
    call "", "print()"
    pop
.endmacro

PRINT "Hello, world!"
```

This allows the programmer to organize macro definitions and other code much more effectively.
