# If* Directives

As stated previously, KASM supports conditional assembly, which means that some code could, or could
not be included in the final binary if logic that KASM follows is met, or not.

These are extremely similar to C-style #if directives.

An example showing a few possible **if*** directives:

```
.define DEBUG
.define VERBOSE 2
.define PRINT_MESSAGES

.ifdef DEBUG                            ; If something is defined
    .if VERBOSE == 1                    ; Nested!
        PRINT "Debug mode on!"
    .elif VERBOSE == 2                  ; Else if
        PRINT "Extra debug mode on!"
    .elifdef PRINT_MESSAGES
        PRINT_MESSAGE("Oh no")
    .endif                              ; Must show end of if
.else
    ; Do something else
.endif
```

In this case the code would end up being this:

```
PRINT "Extra debug mode on!"
```

## Variants

Other variants such as .ifn (if not), .elifn (else if not), and .ifndef/.elifndef also are supported:

* if \<condition>
* ifn \<condition>
* ifdef \<identifier>
* ifndef \<identifier>
* elif \<condition>
* elifn \<condition>
* elifdef \<identifier>
* elifndef \<identifier>
* endif
