
Requirement: Translate assembly code into object files

Options:
    Preprocess then exit

Steps:
    Preprocess
    Parse
    Organize
    Emit

Location counter - Counted by # of instructions, not size
Symbol table

Code is preprocessed
First pass only labels are collected
Second pass things are generated

Each time a symbol is defined, look if it is already defined. If it is, that is an error

Pass 1:
    Read line
    Is a label defined?
        Yes:
            Store name and value
    Add 1 to the LC
    Write back, modified

Support adding constants and labels together

Symbol table, two operations:
    Insert
    Search

Store symbol table as hashes?

Directives:
define
if*
macro
rep
include
line
extern
