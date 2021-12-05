# For Beginners

Hopefully by the time someone is reading this, there will be better options for non-KerbalScript code that
runs inside of kOS. If you are trying to write such a program, or just want to try to learn or practice with
assembly using Kerbal Space Program and kOS, then hopefully this page can give you a brief introduction to assembly language.

If you have previous experience with assembly language, you can probably skip this section and go straight to the [tutorial](../chapter_5/tutorial.md).

For the purposes of this explanation, KerbalScript code will be used to try to illustrate some ideas.

### Code

In most programming languages, when source code is written, it has to undergo a process called compilation. This turns the
code that was written by the user into instructions that the computer can understand. For a lot of programming languages,
there is an intermediary step between the source code, the code written by the user, and the machine code, the binary that
computers can read. This intermediary step is called assembly code.

Take this KerbalScript code for example, that just adds two numbers and stores the result:

```
SET X TO 2 + 4.
```

kOS, or most computers for that matter, have no idea what to do with this, because it is just text. Internally, kOS performs
compilation to turn this code into a form that it can understand.

In this case, the machine code is something like this: (special quirks of compiled kOS machine code may be discussed later)

```
0x4e (2)
0x4e (4)
0x3c
0x34 (X)
```

This may be difficult to understand at first, but we will break it down:

Each line represents what is called an "instruction"

Computers, like kOS, can only do one thing at a time, so it cannot add 2 and 4 together and store them in a variable called X at the same time.

The first instruction has the *opcode* of 0x4e. 0x4e is a *hexadecimal* number. Each instruction is given a unique number that the computer knows,
so it knows what operation to perform. Hexadecimal is just a way of writing this number, but in our normal base-10 number system this number is `78`.
The number itself though, is not important.

The instruction `0x4e` is called the "push" instruction. Inside of kOS there is a bunch of data stacked on top of each other, aptly named the "stack".
The stack is used to store data that is used for something later, but not for storing things long-term like inside of variables.

On the first line, `0x4e (2)` means that we are "pushing" the value of 2 onto the stack. So the number 2 is stored somewhere. This corresponds
to the 2 in our KerbalScript code.

On the second line `0x4e (4)`, we do the same thing, but we push the value of 4. Now the stack contains the numbers 2 and 4.

The third line, opcode `0x3c`, means "add" and it tells the computer to add the last two values on the stack. In this case, that is 2 and 4.
The result is then stored back on the stack, and we know that the answer is 6.

The final line, `0x34 (X)` is opcode `0x34` which tells kOS to store the value on the top of the stack into the variable that is given with the
instruction, in this case that is 'X'.

At the end of this code, the numbers 2 and 4 were added, and the result was stored inside of a variable named X.

This is actually what is happening when the KerbalScript code above is run, and compiled.

### Rationale

KASM is a program which allows the user to directly write these instructions instead of having to write KerbalScript. This can give the user more
control over what their program does, allow them to experiment with learning new things, or to make a compiler which generates KASM assembly code
which can be run inside of kOS, allowing them to create a new programming language for kOS!

A snippet of code in KASM that would perform the same function as the KerbalScript shown above is:
```
push 2
push 4
add
sto "$x"
```

As you can see it can be written as a text file, and yet performs the same function!

How to actually write this code, or the syntax of KASM, is explained in the [tutorial](../chapter_5/tutorial.md).
