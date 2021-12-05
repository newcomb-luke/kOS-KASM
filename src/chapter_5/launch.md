# Simple Launch Code

## INCOMPLETE

This tutorial assumes that you have a built rocket in KSP that you are trying to launch.

In this example we will make a program that will launch a vessel from the launch pad.

This example mainly follows along with the kOS KerboScript documentation.

First, in the last tutorial the kOS terminal prompt and previously run code was still showing up in the terminal.
In order to make it look nicer this time, we will clear the screen.

In KS you would write the code:

```
CLEARSCREEN.
```

This would clear the screen.

It turns out that this is just a built-in function that you call inside of kOS.

We can create a macro that will call it for us, so that all we have to do is invoke that macro:

```
.macro CLEARSCREEN
    push @
    call #, "clearscreen()"
    pop
.endmacro
```

If all we wanted our code to do was clear the screen, then this is all that we would need:

```
.macro CLEARSCREEN
    push @
    call #, "clearscreen()"
    pop
.endmacro

.global _start

.func
_start:
    CLEARSCREEN
    eop
```

The objective here is to write a function that performs a countdown that is printed to the terminal.

In order to help us, we will also declare a helpful macro that will allow us to easily print whatever is on the stack.

```
.macro PRINT
    push @
    swap
    call #, "print()"
    pop
.endmacro
```

This pushes an argument marker, then swaps it with whatever is just below it on the stack (the thing we want to print) and then prints.

Now we can write the code that will allow us to do a countdown:

#### Working Code

```
.macro CLEARSCREEN
    push @
    call #, "clearscreen()"
    pop
.endmacro

.macro PRINT
    push @
    swap
    call #, "print()"
    pop
.endmacro

.global _start

.func
_start:
    CLEARSCREEN

    push "Counting down:"
    PRINT

    push 10 ; We will count down from 10 (counter)

    .countdown_loop:
	dup                  ; Duplicate our counter so that we can compare with it
	push 0               ; We will count down until we reach 0
	swap                 ; Swap the counter value and 0 so that we can compare them
	clt                  ; This pushes true if (counter) <= 0
	btr .countdown_end   ; We jump to the end of the loop if that was true
	dup                  ; Duplicate our counter value on the stack so
                             ;   that we print one, and use the other for counting
	push "..."           ; Push some dots that we will print out before each number
	add                  ; Concatenate "..." + counter
	PRINT                ; Print them

	pushv 1              ; Push a value of 1
	wait                 ; Wait, therefore waits for 1 second

	push 1
	sub                  ; We will subtract 1 from the counter
	jmp .countdown_loop  ; We go through the loop again

    .countdown_end:
	nop
    
    eop
```

