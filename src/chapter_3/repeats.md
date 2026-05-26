# Repeats

A repeat is similar to a macro, but it cannot take arguments. Instead after a `.rep` directive, the preprocessor expects
a number of times for the tokens inside to be repeated. A repeat block is ended with a `.endrep` directive.

## Simple Example

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

## Number of Iterations

You can get access to the current iteration number by specifying either `&0` or `&1`. The only difference between them being that `&0` will start at 0, and `&1` will start at 1.

```
.rep 5
    push &0
.endrep
```

This would expand to the following:

```
push 0
push 1
push 2
push 4
push 4
```

Meanwhile

```
.rep 5
    push &1
.endrep
```

Would expand to the following:

```
push 1
push 2
push 3
push 4
push 5
```

This is just meant to be a quality of life improvement, as you can always perform arithmetic on the result:

```
.rep 5
    push &0 + 3
.endrep
```

```
push 3
push 4
push 5
push 6
push 7
```
