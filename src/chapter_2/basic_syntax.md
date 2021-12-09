# Basic Syntax

Like most assemblers, each KASM source line contains some combination of four fields:

`label: instruction operands ; comment`

Most of these fields are optional, with restriction of the operands field based on which instruction it is.

KASM uses backslash (\\) as a continuation character; if a line ends with a backslash, the next line is considered to be a part of the backslashed line.

Example:

```
push \
     2
```

Would be parsed as the same as:

```
push 2
```

There are no restrictions places on whitespace, such as spaces or tabs, however except for when using the \ character, new lines are to be minded.

## Basic Instructions

A valid source line may simply be:

```
add
```

It contains only the instruction, no label, operands, or comment

A comment is dictated by the character `;` and can be added at the end of any source line and will effectively be ignored.

```
add ; This adds the previous two numbers
```

Instructions that require operands, must be provided operands otherwise it is an error. An example for this could be the `push` instruction:

```
push 2
```

This instruction would push a byte with a value of 2 onto the stack.

Multiple operands are seperated using a comma:

```
bscp 1, 0
```

## Labels

Sometimes in a program, one might want to stop instructions from being executed in order and re-run code, or make a decision based on input,
and run certain code on certain conditions.

Examples of this might be an if-statement or a while loop.

This example demonstrates simply wanting to skip over some code:

```
nop ; Does nothing, just a placeholder
push 2
jmp 3 ; This instruction jumps down 3 instructions, and skips all of them
push 2
add
nop ; This is where we would jump to
```

This would skip the two `push 2` and `add` instructions. This can quickly get messy though, as a typo in the number of instructions to jump
over can cause the entire program to run incorrectly. Thus, labels were created.

Labels are defined before an instruction, and they can either be in the line before it, or in front of it:

```
label:
    nop
also_label: nop
```

Labels can be used in the place of specific places to jump to in the code, that previous program rewritten but using labels is as follows:

```
    nop
    push 2
    jmp the_end
    push 2
    add
the_end:
    nop
```

Now KASM will keep track of the exact value to jump for you, and sections of code can be named, this is illustrated best by a loop:

```
push 5          ; This will loop 5 times
loop:
    dup         ; Duplicates 5 so we can use it for an operation
    push 0      ; We want to compare it against 0
    cgt         ; This will compare the number and 0 and see if it is greater than 0
    bfa end     ; If the number is equal to 0, the loop is over, so we jump to the end
    push 1      ; If not, then we push a 1
    sub         ; And subtract it from the current number
    jmp loop    ; Then start the loop again
end:
    jmp 0       ; Infinite loop
```

Label names can consist of any string of letters and numbers, including underscores. Although they cannot start with a number.

## Inner Labels

All of this example code has existed in isolation, however in larger projects, one would not want to use the label "loop" because
duplicate labels are not allowed. The user could name every single loop something different, but this can get in the way. Inner labels
were created to solve this problem.

Inner labels are tied to a parent label, so they can be reused later in the program if the parent label is different.

Inner labels are differentiated from normal labels because they begin with a `.`

An example of this would be:

```
; This code adds 2 to the variable 'x' five times

adder:              ; This is the parent label
    push 5

    .loop:          ; Inner labels start with a '.'
        dup         ; Looping logic
        push 0
        cgt
        bfa .end
        push 1
        sub

        push "$x"   ; Gets the current value inside of x
        push 1      
        add         ; Adds 1
        sto "$x"    ; Stores the result back into x

        jmp .loop
    .end:
        nop

; Somewhere else in the code
subtracter:
    .loop:          ; This is not a duplicate!
        ...
```
