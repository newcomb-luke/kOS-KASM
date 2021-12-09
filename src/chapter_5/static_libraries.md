# Static Libraries

Static libraries are collections of code that are put all into one file, but can be written in multiple files.
They allow you to reuse code that you have already assembled using `KASM`, so that you don't need to run it each time.

This is where `global` and `external` symbols come into play.

Let's say we have the function from last time called `add_two`, but we want to put it with a bunch of other math functions
(for some reason) so we put it in a file called `math.kasm`:

**math.kasm**
```
.func
add_two:
    add
    ret 0
```

And this is where we call it:

**main.kasm**
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
```

If you tried to assemble `main.kasm` as-is, it would give you the following error:

```
error: instruction references symbol `add_two`, that does not exist
 -->  main.kasm:8:4
  |
8 |     call add_two, #
  |     ^^^^
  |
```

The file `main.kasm` has no way of knowing that the function `add_two` actually exists in another file,
it assumes you will write it in the same file, which we did not.

So you need to tell KASM that it will exist in another file, that it will be "external" to this file.

Therefore we use the `.extern` keyword to declare `add_two` as external:

```
.global _start
.extern .func add_two
```

You note that you also have to specify the type of `add_two` as a function.

`main.kasm` will now assemble! But if you try to run `kld` like in the past to turn it in to a .ksm file,
you will get an error:

```
Unresolved external symbol error. External symbol "add_two" has no definition
```

It says we never defined `add_two`, and that is because we never actually added the code for it!

In order to do that, we need to assemble the file we put `add_two` in:

```
kasm -o math.ko math.kasm
```

Now we have `math.ko`, which we can pass to the linker at the same time as `main.ko`:

```
kld -o program.ksm main.ko add.ko
```

Although... this will give you the same error as before, and that is because we never told KASM to make add_two
available to other files. By default KASM makes every function you have local, so that in theory you can
name a function the same thing twice, and as long as they are local to two different files, then it will work
as expected, each will call their own version, and you won't have to make two different names for them.

To tell KASM to make a function "global" to be seen by other files, we use the `.global` keyword:

```
.global add_two

.func
add_two:
    add
    ret 0
```

Now we have told KASM to make `add_two` global. So now if we run:

```
kasm -o math.ko math.kasm
kld -o program.ksm main.ko math.ko
```

It will not give us an error! And it will work as expected.

You can add as many static libraries as you want to a program, by specifying each one to `kld`:

```
kld -o complex.ksm math.ko main.ko draw.ko files.ko otherstuff.ko ...
```

This will make one giant file named `complex.ksm` that will have all of the code it needs in it.

There are certain times though, that may not want to have one large file, but instead split it up into multiple
files, that we could even possible switch out with future versions and the program that relies on it doesn't need to be
recompiled, a good example would be how regular KerboScript deals with functions in other files.

This is where **shared libraries** come in.
