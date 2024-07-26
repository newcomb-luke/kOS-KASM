## Preprocessor Grammar

### Program

The root node of the entire parse tree

```
<program> ::= <PAST Node> { <PAST Node> }
```

### PAST Node

Preprocessor Abstract Syntax Tree node

```
<PAST Node> ::= <Inert Tokens>
              | <Include>
              | <Single-Line Macro Def>
              | <Multi-Line Macro Def>
              | <Macro Invocation>
              | <Repeat>
              | <If Statement>
              | <Single-Line Macro Undef>
              | <Multi-Line Macro Undef>
```

### Inert Tokens
```
<Inert Tokens> ::= <Inert Tokens> <Inert Tokens>
                 | <Instruction Mnemonic>
                 | <Operator>
                 | <Keyword>
                 | <Type>
                 | <Label>
                 | <InnerLabel>
                 | <InnerLabelReference>
                 | <Literal>
                 | <Whitespace>
                 | <Symbol>
```

### Include
```
<Include> ::= .include <Whitespace> <StringLiteral> <Newline>
            | .include <Whitespace> <Macro Invocation> <Newline>
```

### Single-Line Macro Definition
```
<Single-Line Macro Def> ::= .define <Identifier> <Newline>
                          | .define <Identifier> <SLMacro Def Contents> <Newline>
                          | .define <Identifier> (<SLMacro Def Arguments>) <Newline>
                          | .define <Identifier> (<SLMacro Def Arguments>) <SLMacro Def Contents> <Newline>
```

### Single-Line Macro Definition Arguments
```
<SLMacro Def Arguments> ::= <Identifier> | {, <Identifier>}
```

### Single-Line Macro Definition Contents
```
<SLMacro Def Contents> ::= { <Identifier>
                           | <Literal>
                           | <Operator>
                           | <Keyword>
                           | <Type>
                           | <Label>
                           | <InnerLabel>
                           | <InnerLabelReference>
                           | <Symbol>
                           | <Macro Invocation> }
```

### Multi-Line Macro Definition
```
<Multi-Line Macro Def> ::= .macro <Identifier> <MLMacro Def Args> <Newline> .endmacro
                         | .macro <Identifier> <MLMacro Def Args> <Newline> <Multi-Line Macro Contents> .endmacro
```

### Multi-Line Macro Definition Arguments
```
<MLMacro Def Args> ::= <Literal Integer>
                     | <Literal Integer>-<Literal Integer> <MLMacor Def Defaults>
```

### Multi-Line Macro Definition Argument Defaults
```
<MLMacro Def Defaults> ::= <MLMacro Def Default> {, <MLMacro Def Default>}
```

### Multi-Line Macro Definition Argument Default
```
<MLMacro Def Default> ::= <MLMacro Def Default> <MLMacro Def Default>
                 | <Instruction Mnemonic>
                 | <Operator>
                 | <Keyword>
                 | <Type>
                 | <Label>
                 | <InnerLabel>
                 | <InnerLabelReference>
                 | <Literal>
                 | <Non-Newline Whitespace>
                 | <Non-Comma Symbol>
```

### Multi-Line Macro Definition Contents
```
<Multi-Line Macro Contents> ::= <PAST Node> | <MLMacro Arg Reference>
                                { <PAST Node> | <MLMacro Arg Reference> }
```

### Macro Invocation
```
<Macro Invocation> ::= <Identifier>
                     | <Identifier> ()
                     | <Identifier> (<Macro Invocation Arguments>)
```

### Macro Invocation Arguments
```
<Macro Invocation Arguments> ::= <Macro Invocation Arg> {, <Macro Invocation Arg>}
```

### Macro Invocation Argument
```
<Macro Invocation Arg> ::= <Instruction Mnemonic>
                         | <Macro Invocation>
                         | <MLMacro Arg Reference>
                         | <Operator>
                         | <Keyword>
                         | <Type>
                         | <Label>
                         | <InnerLabel>
                         | <InnerLabelReference>
                         | <Literal>
                         | <Non-Newline Whitespace>
                         | <Non-Comma Symbol>
```

### Repeat
```
<Repeat> ::= .rep <Expression> <Newline> .endrep
           | .rep <Expression> <Newline> <Repeat Contents> .endrep
```

### Repeat Contents
```
<Repeat Contents> ::= <PASTNode> { <PASTNode> }
```

### If Statement
```
<If Statement> ::= <If Begin> <If Contents> { <Middle If> <If Contents> } .endif
                 | <If Begin> <If Contents> { <Middle If> <If Contents> } .else <If Contents> .endif
```

### If Statement Beginning
```
<If Begin> ::= <Starting If> <Expression> <Newline>
```

### If Statement Starting Directive
```
<Starting If> ::= .if .ifn .ifdef .ifndef
```

### If Statement Middle Directive
```
<Middle If> ::= .elif .elifn .elifdef .elifndef
```

### If Statement Contents
```
<If Contents> ::= <PASTNode> { <PASTNode> }
```

### Single-Line Macro Undefinition
```
<Single-Line Macro Undef> ::= .undef <Identifier>
                            | .undef <Identifier> <Literal Integer>
```

### Multi-Line Macro Undefinition
```
<Multi-Line Macro Undef> ::= .unmacro <Identifier>
                           | .unmacro <Identifier> <Literal Integer>
                           | .unmacro <Identifier> <Literal Integer>-<Literal Integer>
```