# Full Instruction List

This is simply a list of all instructions that are in KASM, grouped by common purpose.

## Stack Operations

**push** (any) - Pushes a value to the stack

**pushv** (any) - Pushes a value to the stack, but as kOS "scalar" or "value" versions  
(this instruction does not exist in kOS, it is just for the purposes of user control)

**pop** - Removes the value from the top of the stack and discards it

**dup** - Copies the value from the top of the stack and pushes that copy, so there are two of the original

**swap** - Swaps the value on the top of the stack and the one below it

**eval** - Returns the value referenced by the topmost variable on the stack

**targ** - Asserts that the topmost thing on the stack is an argument marker (throws a kOS error if it isn't!)

## Mathematical Operations

These all push the result back to the stack

All of these behave in this way:

(this side is the value just below the top of the stack) - (this is the top of the stack)

**add** - Adds the two arguments on top of the stack

**sub** - Subtracts the two arguments on top of the stack

**mul** - Multiplies the two arguments on top of the stack

**div** - Divides the two arguments on top of the stack

**pow** - Raises the value just under the top, to the power of the top

**neg** - Pushes back the negative of the number on the top of the stack

## Logical Operations

All of these behave in this way:

(this side is the value just below the top of the stack) > (this is the top of the stack)

**cgt** - Checks if one number is greater than the other

**clt** - Less than

**cge** - Greater than or equal

**cle** - Less than or equal

**ceq** - Equal to

**cne** - Not equal to

**not** - Pushes back the opposite of the boolean value on top of the stack (true -> false, false -> true)

**and** - Pushes true, if both values on top of the stack are true

**or** - Pushes true, if at least one value on top of the stack is true

## Flow Control

**eof** - Tells kOS that this is the end of the file, so it should stop reading the program

**eop** - Tells kOS that this is the end of the program

**nop** - No-Op, does nothing

**jmp** (string | int | label) - Unconditionally jumps to the location specified in the operand, it can be a string or an integer (for relative jumps)

**call** (string | label), (string) - Calls a subroutine, leaving the result on the stack, both operands are strings

**ret** (int) - Returns from a subroutine and pops off the number of scopes as provided in the operand

**btr** (string | int | label) - If the value on the top of the stack is true, then it branches to the given location

**bfa** (string | int | label) - Same as btr but if the value is false

**wait** - Pops a duration in seconds from the stack and waits for that amount of game time

## Variable Manipulation

**sto** (string) - Stores the value on the top of the stack in the variable specified by the operand

**uns** - Removes the variable named by the value on the top of the stack

**stol** (string) - Stores the value on the top of the stack in the variable specified by the operand, but if the variable does not exist in the current scope it won't attempt to look for it in a higher scope.

**stog** (string) - Stores the value on the top of the stack in the variable specified by the operand, but always stores it in a global variable.

**stoe** (string) - Stores the value on the top of the stack in the variable specified by the operand, but if the variable does not exist yet, it will throw an error instead of creating a new one.

**exst** - Tests if the identifier on the top of the stack is a variable, and is in scope. It will push true if it is, and false if it is not.

## Member Values

**gmb** (string) - Consumes the topmost value of the stack, getting the suffix of it specified by operand and putting that value back on the stack. Ex: `SHIP:ALTITUDE`

**smb** (string) - Consumes a value and a destination object from the stack, setting the objects suffix specified by the operand to the value.

**gidx** - Consumes an index and an target object from the stack, getting the indexed value from the object and pushing the result back on the stack.

**sidx** - Consumes a value, an index, and an object from the stack, setting the specified index on the object to the given value.

**gmet** - Similar to `gmb` except instead of getting the member as a value, it is called as if it was a function.

Ex: `SHIP:NAME()`

## Triggers

**addt** (bool), (int) - Pops a function pointer from the stack and adds a trigger that will be called each cycle. The second operand
contains the Interrupt Priority level of the trigger. For one trigger to interrupt another, it needs a higher priority, else it waits
until the first trigger is completed before it will fire.

**rmvt** - Pops a function pointer from the stack and removes any triggers that call that function pointer.

## Scopes

**bscp** (int), (int) - Pushes a new variable namespace scope (for example, when a "{" is encountered in a block-scoping language),
the first operand being the new scope id, and the second one being the parent scope id.

**escp** (int) - Pops a variable namespace scope (for example, when a "}" is encountered in a block-scoping language), the operand
being the number of scope depths to pop.

## Delegates

**phdl** (int), (bool) - Pushes a delegate object onto the stack, optionally capturing a closure.

**prl** (string) - See KOS Code docs for more information.

**pdrl** (string), (bool) - This serves the same purpose as prl, except it's for use with UserDelegates instead of raw integer IP calls.

## Misc.

**lbrt** (string) - It exists purely to store, as an operand, the label of the next opcode to follow it. Mostly used for internal kOS things.  
This shouldn't really be used when writing KASM unless you know exactly what you are doing, as it can have unintended and hard to debug consequences.

**tcan** - Tests whether or not the current subroutine context on the stack that is being executed right now is one that has been flagged
as cancelled by someone having called SubroutineContext.Cancel(). This pushes a True or a False on the stack to provide the answer. This should be
the first thing done by triggers that wish to be cancel-able by other triggers.
