# kOS-KASM

## The Kerbal Compiler Collection assembler for kOS

KASM is a well-featured assembler that works with KLinker to assemble code to run in the kOS mod for Kerbal Space Program.

## Documentation / Tutorials

https://newcomb-luke.github.io/kOS-KASM/

## Sample Usage

kasm input.kasm -o program.ko

### Feature list:

- [x] Parsing KASM
    - [x] Lexer/Tokenizer
    - [x] Preprocessor
        - [x] Definitions
            - [x] Parsing
            - [x] Expansion
        - [x] Macros
            - [x] Parsing
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
        - [x] Instruction parser
- [x] First pass
- [x] Second pass
    - [x] Generating KO files
