# Hello World

Following in the footsteps of the official kOS KerboScript tutorials, the first thing that you will learn is how to print "Hello world"

There is some boilerplate (template) code that one must always write to create an executable program in KASM:

Start off by creating a new file called `hello.kasm` wherever you would like.

Then write the following lines:

```
.global _start

.func
_start:
    eop
```

This is actually the smallest valid KASM program that one can write.

#### Code Breakdown

```
.global _start
```

This declares a global symbol called `_start`. `_start` is a special symbol name that is reserved for a function.
This function is where the program starts, hence the name. This must be global or else the linker will not be able to find it
and will give you an error.

```
.func
_start:
    eop
```

This declares a function, named `_start` which only contains one instruction: `eop`

`eop` tells kOS that the code is over, and this instruction should ***always*** go at the end of your `_start` or `_init` functions.
Otherwise, strange things happen.

Now we can resume writing our hello world program!

All we do is add the following four lines inside of our `_start`:

```
push @
push "Hello world!"
call #, "print()"
pop
```

When calling a function in KASM, there are two ways to do it:

For functions that you have created:

```
call func_name, #
```

And:

```
call #, "func_name()"
```

The above syntax is used when calling built-in kOS functions. There will be a list of those at the end of the tutorials list.

When calling built-in functions, if they are not meant to return any value to you, such as `print()`, then they return a NULL value.

In order for the stack to not be filled up with them we add a `pop` instruction after `call` to just throw it away.

Both ways of calling a function require you to `push` your arguments onto the stack before you call it.

First we `push @` to push a function *argument marker* which just is there to mark the end of the arguments to a function. Remember
`push` adds things to the top of the stack.

So when we next say `push "Hello world!"` with actually pushes the string we want to print onto the stack.

The stack now has our string, and our argument marker. Then `call` is called.

#### Final code

The final code to print out hello world onto the screen in kOS is:

```
.global _start

.func
_start:
    push @
    push "Hello world!"
    call #, "print()"
    pop
    eop
```

Feel free to replace the string `"Hello world!"` with whatever you want.

You can even replace the string entirely, `print()` prints numbers and booleans as well.

#### Using the code

In order to get this code into a format kOS can use, we have to run these two commands:

```
kasm -o hello_world.ko hello_world.kasm
kld -o hello_world.ksm hello_world.ko
```

Now you have a file named `hello_world.ksm` that you can put into your `Ships/Script` folder
that you can run in kOS by typing:

```
SWITCH TO 0.
RUN hello_world.ksm.
```

See [how to run KASM](../chapter_1/running_kasm.md) and the [KLinker Docs](https://github.com/newcomb-luke/kOS-KLinker) if you are confused.
