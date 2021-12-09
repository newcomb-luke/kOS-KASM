# Shared Libraries

Shared libraries are useful because you don't have to link all code in your project over and over whenever
the code changes. It also allows you to keep file sizes smaller. The same shared library can be used by multiple
programs at the same time, and only one copy of it has to exist.

# In KerboScript

As you probably know, in KerboScript when you want to load the code from another file, all you have to do us "run"
that file. If we wanted the code to only be loaded once (because it is just a bunch of functions) we could do something
like:

```
RUNONCEPATH("library.ksm").
```

And that would load our functions that we can later call. In order to understand how to do something similar in KASM, we
must first look at the [disassembly](https://en.wikipedia.org/wiki/Disassembler) of calling `RUNONCEPATH("library.ksm").`

This can be found by first compiling the file, in this case the file with the above code was called `main.ks` and then
running [KDump](https://github.com/newcomb-luke/KDump) (`kdump --disassembly main.ksm`):

```
push  @
push  @
push  true
pushv  "library.ksm"
eval
call  "@LR00",#
pop
```

This may be confusing at first, but mostly all that is happening here is that there is a place in the code called "@LR00"
which is loaded into kOS every time you run a program from the terminal. It exists so that each and every piece of user
code doesn't need to have all the code in it required to run a program every time the user wants to.

This code is able to be changed by the kOS devs at any time because it is not meant to be written directly by users, and is
always accessed by the "run" family of functions. In order to understand what we are calling however, as of kOS version 1.3.0.0,
the code is:

```
bscp -999, 0     ; This instruction has the label "@LR00"
stol "$runonce"
stol "$filename"
push @
push "$filename"
eval
push true
push #
call #, "load()"
bfa run_program
push "$runonce"
bfa run_program
pop_loop:
    pop
    argb
    btr after_pop
    jmp pop_loop
after_pop:
    pop
    push 0
    ret 1
run_program:
    stol "$entrypoint"
    call "$entrypoint", #
    pop
    push 0
    ret 1
```

This is a whole lot to digest, but basically there is a built-in function called `load()` that kOS calls when it wants to load
a file. It returns various things, in particular the program's "entry point" which is called like a function, and that is just
the first instruction in the program you are trying to run.

The arguments we are more interested in though, and on lines 2 and 3, there are the `stol` instructions which store if the program
should only be run once, and the other one stores the filename. So these two things are the parameters to the kOS loader.

# Loading

In order to do the same thing in our program, we can simply make a piece of code that calls "@LR00" the same way the KerboScript code does.

This is not too difficult to do. Here we create a macro to do it for us just in case we want to load multiple:

```
.global _start

.macro LOAD 1
    push @
    push @
    push true
    pushv &1
    eval
    call "@LR00", #
    pop
.endmacro

.func
_start:
    LOAD "library.ksm"
    eop
```


