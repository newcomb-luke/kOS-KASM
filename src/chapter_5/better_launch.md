# Better Launch Code

This tutorial builds off of the last, improving our launch code.

The first order of business is to actually start the engine. As you know this is done by setting the throttle to 100%, and staging.

In order to set the throttle, we need to introduce how to access variables in KASM.

#### Throttle

Variables exist in a scope, which has an ID, which can be set up by two instructions:

* bscp - Begin scope
* escp - End scope

By default kOS creates a global variable scope, whose ID is 0. If we want to make variables in our "main function" scope, the
scope ID is incremented, for example to create our variable scope will be 1:

```
bscp 1, 0

... our code

escp 1
```

The first operand to `bscp` is the new scope ID, and the second is supposed to be the "parent ID", or the scope that this scope is in.

The operand to `escp` is the scope ID that we are ending. This removes any variables that were declared in that scope.

If you know C-style languages, or even just KerboScript, you can think of `bscp` as a `{` and `escp` as a `}`

Now that we have that, in order to set the throttle, all we have to do is set the value of a variable named `throttle`. Variables in 
kOS are prefixed with the `$` character, so the name would be `$throttle`

In order to store a value on the stack to a variable, there is the `sto` instruction, which stands for store.

So in order to set the throttle, the code is:

```
bscp 1, 0

...

push 1.0 ; The value we want the throttle to be
sto "$throttle"

... other code

escp 1
```

#### Staging

When you write the KS code `STAGE.`, it actually just calls a built-in function: `stage()`

So this is the code you have to write to stage in KASM:

```
push @
call #, "stage()"
pop
```

#### Steering

We need to do the equivalent of KerboScript `SET STEERING TO UP.`

Once again, this is done using global variables.

The `UP` is actually a global variable that contains a rotation value that is straight up.

Steering is similar to throttle in that there is a global variable named `$steering` that we set
to the desired rotation.

Therefore setting the steering to up can be performed using the following code:

```
push "$up"
sto "$steering"
```

Note that `push`-ing a variable puts the variable's value on the stack

#### Waiting

If the code were to set the throttle, set the steering, and then end, kOS would tell the throttle and steering
to go back to normal. Therefore in order to make them stay, we need the program to not end.

This can be performed with an infinite loop like so:

```
.infinte:
    push 0
    wait
    jmp .infinte
```

Note the use of `push 0` and `wait`. As noted in the kOS documentation, if you just have an infinite loop
it can slow down your game, and use more electric charge. Therefore we wait for 0 seconds, to provide a small but unnoticable delay.

This would work perfectly fine, but to be a little more sophisticated, we will
follow more along with the kOS tutorial and wait until we are above 70,000 m.

Similar to throttle, steering, and up, the `SHIP` variable it self is just a global variable named `$ship`.

In KerboScript in order to get the ship's altitude, one would write: `SHIP:ALTITUDE`

In order to do the `:`, we will use an instruction called `gmb`, which stands for "get member"

To get the ship's altitude, we would write:

```
push "$ship"
gmb "altitude"
```

To check this in a loop against 70,000 we would write:

```
.altitude_loop:
    push 0              ; Wait 0 seconds to provide a little delay
    wait
    push "$ship"
    gmb "altitude"      ; SHIP:ALTITUDE
    push 70000          ; > 70,000
    cgt                 ; Actually performs the comparison
    bfa .altitude_loop  ; If it isn't 70,000 yet, loop again
```

## Putting it all together

Combining all of the code we have written so far together, the result would be:

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
    bscp 1, 0

    CLEARSCREEN

    ; Do the countdown
    push "Counting down:"
    PRINT
    push 10
    .countdown_loop:
	dup
	push 0
	clt
	btr .countdown_end
	dup
	push "..."
	swap
	add
	PRINT
	pushv 1
	wait
	push 1
	sub
	jmp .countdown_loop
    .countdown_end:

    ; Set the throttle
    push 1.0
    sto "$throttle"

    ; Set steering
    push "$up"
    sto "$steering"

    ; Stage
    push @
    call #, "stage()"
    pop

    ; Wait to reach 70km

    .altitude_loop:
	push 0              ; Wait 0 seconds to provide a little delay
	wait
	push "$ship"
	gmb "altitude"      ; SHIP:ALTITUDE
	push 70000          ; > 70,000
	cgt                 ; Actually performs the comparison
	bfa .altitude_loop  ; If it isn't 70,000 yet, loop again

    escp 1
    eop
```

