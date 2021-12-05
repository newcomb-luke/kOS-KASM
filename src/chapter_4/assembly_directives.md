# Chapter 4: Assembly Directives

Besides the preprocessor directives, KASM also has a few assembly directives that are not for preprocessing purposes.

## `.extern` - External

This directive defines a symbol as external to the linker.

This means that whatever symbol this specifies is not in this file, but is in another file that will be linked with the output of this one.

In more general terms, it is a promise that a symbol exists somewhere else.

#### Example

```
.extern .func add_func

call add_func, #
```

This declares an extenal function (`.func`) called `add_func`, and then calls that function later.

## `.global` - Global

This directive is similar to `.extern` but it declares that a symbol *will* be in this file, but it will also let it be visible to other files.
Each `.extern` needs to be paired with a `.global` in another file.

```
.global _start

.func
_start:
    ...
```

## `.local` - Local

This declares a symbol as local, which is the default. This is only provided to be thorough.

```
.local func_name ; This is redundant

.func
func_name:
    ...
```

## `.type` - Symbol Type

This declares a symbol's type. As you may have noticed in the `.extern` example there was the type of the symbol specified (`.func`).

This can be done when the symbol's binding (local, global, or extern) is specified, but it can also be done later.

An equivalent set of code to the `.extern` example, but using `.type` would be:

```
.extern add_func
.type .func add_func
```

To define an external symbol that is a value and not a function, just use the `.value` type:

```
.extern number
.type .value number
```

## `.func` - Function

This directive is used to mark a certain label as a function, which is treated differently in KASM.

Otherwise KASM would have no idea if you want a label to be inside the last function, or a new one.

The usage of the `.func` directive is as such:

```
...

.func
add_func:
    ....
```
