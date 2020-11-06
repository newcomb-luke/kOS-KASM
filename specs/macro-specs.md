
# KASM Macros

.define CONSTANT VALUE

.define PUSH2 push 2

.define a(x)    1 + b(x)
.define b(x)    2*x

push.i16        a(8) ; expands to 1 + 2 * 8

.define foo (a, b) ; no arguments, (a, b) is the expansion
.define bar(a, b)  ; two arguments, empty expansion

; detect recursive macro expansions

; Macro overloading
.define foo(x)    1 + x
.define foo(x, y) 1 + x + y

; Macro redefinition

.undef foo

.assign i i+1

.macro  some_macro 1

push 1
push &1
add

.endmacro

; A macro with default parameters
.macro RET 0-1 1

ret &1

.endmacro

.unmacro RET 1 ; removes nothing because there is no RET macro with exactly one parameter

.if
.ifn
.elif
.elifn
.else
.endif

.ifdef
.ifndef

.ifdef DEBUG
    push  ARGM
    push  "Meep"
    call ,"print()"
.endif

; repeats some stuff 12 times
.rep 12

.endrep

.include "somefilename.andsomeextension"

.line 2

; $ means the address of the current instruction

### Unary
- negate
~ flip
! not

### Mathematical

+
-
*
/

%

### Comparison
== / !=
<
<=
\>
\>=

### Logical
&& - logical and
|| - logical or

?/: - ternary

; Local labels  
; .inner_loop

label1

.loop

label2

.loop

; decimal
200

; hex
0xaf

; binary
0b1100_0001

    OPENPAREN,
    CLOSEPAREN,
    IDENTIFIER(&'source str),
    INT(i32),
    DOUBLE(f64),
    MINUS,
    COMP,
    NEGATE,
    ADD,
    MULT,
    DIV,
    AND,
    OR,
    EQ,
    NEQ,
    LT,
    LTE,
    GT,
    GTE,
    QUESTION,
    COLON,
    NEWLINE