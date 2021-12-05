# Repeats

A repeat is similar to a macro, but it cannot take arguments. Instead after a `.rep` directive, the preprocessor expects
a number of times for the tokens inside to be repeated. A repeat block is ended with a `.endrep` directive.

## Example

```
.rep 5
    push 0
.endrep
```

This would expand to the following:

```
push 0
push 0
push 0
push 0
push 0
```
