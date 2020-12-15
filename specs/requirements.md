
Requirement: Translate assembly code into object files

Options:
    Preprocess then exit

Steps:
    Preprocess
    Parse
    Organize
    Emit

Directives:
define
if*
macro
rep
include
line
extern
func

Location counter - Counted by # of instructions, not size
Label table

Code is preprocessed
First pass only labels are collected
Second pass things are generated

Each time a Label is defined, look if it is already defined. If it is, that is an error

Pass 1:
    Read line
    Is a label defined?
        Yes:
            Store name and value
    Add 1 to the LC
    Write back, modified

Label table, two operations:
    Insert
    Search

Store Label table as hashes?
