# kOS-KASM

## The Kerbal Compiler Collection assembler for kOS

This is not meant to be used by anything yet in this condition!

### Feature list:

- [ ] Parsing KASM
    - [x] Lexer/Tokenizer
    - [x] Preprocessor
        - [x] Definitions
            - [x] Parsing
            - [x] Expansion
        - [x] Macros
            - [x] Parsing
                - [ ] Ability to use definitions and expressions as macro argument numbers?
            - [x] Expansion
        - [x] Rep
        - [x] Include
        - [x] Extern
        - [x] Global
        - [x] If*
        - [x] Undef
        - [x] Unmacro
        - [x] Func
        - [ ] Line
    - [x] Parser
        - [x] Expression parser
        - [x] Expression evaluator
        - [ ] Instruction parser
- [ ] First pass
- [ ] Second pass
    - [ ] Generating KO files