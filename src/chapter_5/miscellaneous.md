# Miscellaneous

Now that we have introduced many of the aspects of KASM and inner workings of kOS, this page will
serve as a quick reference for implementing various things.

## If statements

Let's say we had the variable `$x`, and wanted to basically do:

```
IF X > 1 {
    PRINT "IF".
} ELSE {
    PRINT "ELSE".
}
```

The KASM implementation would be:

```
	push "$x"
	push 1
	cgt         ; x > 1
	bfa .else
	push @
	push "IF"
	call #, "print()"
	pop
	jmp .if_end
    .else:
	push @
	push "ELSE"
	call #, "print()"
	pop
    .if_end:
	...
```

## While loop

Let's say we wanted to implement the following while loop (the inverse of an `UNITL` loop):

```
while SHIP:ALTITUDE < 70000 {
    wait 0.
}
```

This would be implemented in KASM as:

```
.loop:
    push 0
    wait
    push "$ship"
    gmb "altitude"
    push 70000
    clt            ; SHIP:ALTITUDE < 70000
    btr .loop
```

## For loop

Example code in C style:

```
for (int i = 0; i < 10; i++) {
    PRINT i.
}
```

In KerboScript:

```
FROM {local i is 0.} UNTIL !(i < 10) STEP {SET i to i + 1.} DO {
    PRINT i.
}
```

KASM (using the stack):

```
push 0
.loop:
    dup
    push 10
    clt
    bfa .loop_end
    dup
    push @
    swap
    call #, "print()"
    pop
    push 1
    add
    jmp .loop
.loop_end:
    ...
```

KASM (using a kOS variable `$i`):

```
bscp 1, 0
push 0
stol "$i"
.loop:
    push "$i"
    push 10
    clt
    bfa .loop_end
    push @
    push "$i"
    call #, "print()"
    pop
    push "$i"
    push 1
    add
    sto "$i"
    jmp .loop
.loop_end:
    escp 1
```
