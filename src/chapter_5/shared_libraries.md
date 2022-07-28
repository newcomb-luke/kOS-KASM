# Shared Libraries

Shared libraries are useful because you don't have to link all code in your project over and over whenever
the code changes. It also allows you to keep file sizes smaller. The same shared library can be used by multiple
programs at the same time, and only one copy of it has to exist.

## In KerboScript

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
    targ
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

## Loading

In order to do the same thing in our program, we can simply make a piece of code that calls "@LR00" the same way the KerboScript code does.

This is not too difficult to do. Here we create a macro to do it for us just in case we want to load multiple:

```
.global _start

.macro LOAD 1
    push @
    push @
    push true
    pushv &1 ; &1 gets replaced with the argument we give
    eval
    call "@LR00", #
    pop
.endmacro

.func
_start:
    LOAD("library.ksm)
    eop
```

This program will "load" (aka run) our `library.ksm` file.

## Delegates

In KerboScript, delegates are functions that you can effectively store in a variable. As of the current version of KerboScript,
it turns out that all functions are actually used in the same way that delegates are. This allows you to assume that a function exists
and generate `call` instructions for it just by using a variable name. The way that you would call a function like this would be:

```
call #, "$add_two*"
```

Because it is a function delegate and not just a normal variable, the convention is to put a ***\**** after the variable name. In this
case we call the function `add_two` because that is the name of it in our code, but you can mix and match the names although that might
be confusing.

This would replace the normal:

```
call add_two, #
```

That we would do if calling the function statically.

## Library Side

Now that we know how to call the function that we have loaded, we need to know how to set that up from the library's view.

This is where the special `_init` function label comes into play in KASM. It is used to initialize things inside of a shared library,
which usually takes the form of setting up function delegates.

An example of the shared library code for our `add_two` function would be:

```
.global _init

.func
_init:
    pdrl add_two, true
    sto "$add_two*"

    push @
    push "Loaded!"
    call #, "print()"
    pop

    ret 0 ; Note this, see below


.func
add_two:
    add
    ret 0
```

As you can see, our code for `add_two` stays exactly the same, but we have the new `_init` function.

Here we use the special `pdrl` instruction. This stands for "push delegate relocate later", and basically allows us to store a function
onto the stack, and then store it in a variable. The first argument is the function that we want to push, and the second argument is a
boolean of whether or not we would like to capture a closure or not.

Then the next instruction we do is just storing our delegate into `"$add_two*"` like above.

Then just to verify that it is working, for fun we print out "Loaded!".

**Note**: Notice at the bottom of `_init` we `ret 0`. This may be confusing, but remember in the kOS file loader, it simply calls the
file as a function, so in order to tell kOS that our `_init` function is over, we need to `ret`, and in this case we haven't created any
variable scopes.

The main program can be linked normally (`kld -o main.ksm main.ko`) because it doesn't do anything special. But KLinker needs slightly more
information in order to tread the shared library as it would need to be treated.

So in order to link a shared library, all you have to change is to add the **--shared** flag:

```
kld --shared -o library.ksm library.ko
```

Then you will have two files named `main.ksm` and `library.ksm` that when you put in the same place, you can run `main.ksm` and it will
call functions inside of `library.ksm`.

## Final code

Here is the final code in `main.kasm`:

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
	LOAD("library.ksm")

	push @
	push @
	push 2
	push 3
	call #, "$add_two*"
	call #, "print()"
	pop
	eop
```

And `library.kasm`:

```
.global _init

.func
_init:
	pdrl add_two, true
	sto "$add_two*"

	push @
	push "Loaded!"
	call #, "print()"
	pop
	ret 0

.func
add_two:
	add
	ret 0
```

## Calling KerboScript Functions

Because you have previously seen how KerboScript function calling actually works (using `pdrl`), it is worth noting that this is both how
you can call KerboScript functions from your KASM code, and how you can make KASM functions callable from KerboScript code.

## Final Notes

This is the last offical "tutorial" section of this book. The following section only lists the various kOS built-in functions. You now
have enough information to do whatever you desire to do in KASM.

If after reading the list of all built-in functions, you are still confused on how to do something in KASM, I encourage you to write
equivalent code in KerboScript, and then run [KDump](https://github.com/newcomb-luke/KDump) in order to see how KerboScript does it.

If there are requests to create a new tutorial section for something that is more specific, then I will definitely consider adding it.
