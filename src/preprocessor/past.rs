use std::num::NonZeroU8;

use crate::errors::Span;
use crate::lexer::Token;

/// PAST stands for Preprocessor Abstract Syntax Tree
///
/// Basically, in KASM the preprocessor is treated as a tiny programming language, and is first
/// parsed, then "generated" which means that it generates the rest of the code that will be used
/// in KASM's subsequent operation.
///

#[derive(Debug)]
pub enum PASTNode {
    BenignTokens(BenignTokens),
    SLMacroDef(SLMacroDef),
    MacroInvok(MacroInvok),
    MLMacroDef(MLMacroDef),
    SLMacroUndef(SLMacroUndef),
    MLMacroUndef(MLMacroUndef),
    Repeat(Repeat),
    IfStatement(IfStatement),
    Include(Include),
}

impl PASTNode {
    pub fn span_end(&self) -> usize {
        match self {
            PASTNode::BenignTokens(benign_tokens) => benign_tokens.span.end,
            PASTNode::SLMacroDef(sl_macro_def) => sl_macro_def.span.end,
            PASTNode::MacroInvok(macro_invok) => macro_invok.span.end,
            PASTNode::MLMacroDef(ml_macro_def) => ml_macro_def.span.end,
            PASTNode::SLMacroUndef(sl_macro_undef) => sl_macro_undef.span.end,
            PASTNode::MLMacroUndef(ml_macro_undef) => ml_macro_undef.span.end,
            PASTNode::Repeat(repeat) => repeat.span.end,
            PASTNode::IfStatement(if_statement) => if_statement.span.end,
            PASTNode::Include(include) => include.span.end,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Ident {
    pub span: Span,
    pub hash: u64,
}

impl Ident {
    pub fn new(span: Span, hash: u64) -> Self {
        Self { span, hash }
    }
}

impl PartialEq for Ident {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

#[derive(Debug)]
pub struct BenignTokens {
    pub span: Span,
    pub tokens: Vec<Token>,
}

impl BenignTokens {
    /// Creates a new BenignTokens struct using the tokens provided
    ///
    /// The vector MUST NOT BE EMPTY. If it is, this function will panic
    ///
    pub fn from_vec(tokens: Vec<Token>) -> Self {
        let mut span = Span::new(0, 0, 0);

        let first_span = tokens.first().unwrap().as_span();
        let last_span = tokens.last().unwrap().as_span();

        span.file = first_span.file;
        span.start = first_span.start;
        span.end = last_span.end;

        Self { span, tokens }
    }
}

/// A PAST Node representing a single line macro definition
///
/// Grammar:
///
/// ```sh,ignore,no_run
/// <SLMacroDef> ::= .define <identifier>
///              |   .define <identifier> <SLMacroDefContents>
///              |   .define <identifier> <SLMacroDefArgs>
///              |   .define <identifier> <SLMacroDefArgs> <SLMacroDefContents>
/// ```
///
#[derive(Debug)]
pub struct SLMacroDef {
    pub span: Span,
    pub identifier: Ident,
    pub args: Option<SLMacroDefArgs>,
    pub contents: Option<SLMacroDefContents>,
}

impl SLMacroDef {
    pub fn new(
        span: Span,
        identifier: Ident,
        args: Option<SLMacroDefArgs>,
        contents: Option<SLMacroDefContents>,
    ) -> Self {
        SLMacroDef {
            span,
            identifier,
            args,
            contents,
        }
    }
}

/// A PAST Node representing a single line macro definition's arguments
///
/// Grammar:
///
/// ```sh,ignore,no_run
/// <SLMacroDefArgs> ::= ()
///                  |   (<arguments>)
///
/// <arguments> ::= <identifier> | <identifier>, <arguments>
/// ```
///
#[derive(Debug)]
pub struct SLMacroDefArgs {
    pub span: Span,
    pub args: Vec<Ident>,
}

impl SLMacroDefArgs {
    pub fn new(span: Span, args: Vec<Ident>) -> Self {
        Self { span, args }
    }
}

/// A PAST Node representing a single line macro definition's contents
///
/// This grammar may be incomplete, however it is meant to convey that this can contain anything
/// except any preprocessor directives.
///
/// Grammar:
///
/// ```sh,ignore,no_run
/// <SLMacroDefContents> ::=
///                      |   <identifier> <SLMacroDefContents>
///                      |   <literal> <SLMacroDefContents>
///                      |   <non-definition directive> <SLMacroDefContents>
///                      |   <operator> <SLMacroDefContents>
///                      |   <keyword> <SLMacroDefContents>
/// ```
///
#[derive(Debug)]
pub struct SLMacroDefContents {
    pub span: Span,
    pub contents: Vec<PASTNode>,
}

impl SLMacroDefContents {
    pub fn new(span: Span, contents: Vec<PASTNode>) -> Self {
        Self { span, contents }
    }
}

#[derive(Debug)]
pub struct MacroInvok {
    pub span: Span,
    pub identifier: Ident,
    pub args: Option<MacroInvokArgs>,
}

impl MacroInvok {
    pub fn new(span: Span, identifier: Ident, args: Option<MacroInvokArgs>) -> Self {
        Self {
            span,
            identifier,
            args,
        }
    }
}

#[derive(Debug)]
pub struct MacroInvokArgs {
    pub span: Span,
    pub args: Vec<MacroInvokArg>,
}

impl MacroInvokArgs {
    pub fn new(span: Span, args: Vec<MacroInvokArg>) -> Self {
        Self { span, args }
    }

    pub fn from_vec(args: Vec<MacroInvokArg>) -> Self {
        let mut span = Span::new(0, 0, 0);

        let first_span = args.first().unwrap().span;
        let last_span = args.last().unwrap().span;

        span.start = first_span.start;
        span.file = first_span.file;
        span.end = last_span.end;

        MacroInvokArgs { span, args }
    }
}

#[derive(Debug)]
pub struct MacroInvokArg {
    pub span: Span,
    pub contents: Vec<PASTNode>,
}

impl MacroInvokArg {
    pub fn new(span: Span, contents: Vec<PASTNode>) -> Self {
        Self { span, contents }
    }
}

#[derive(Debug)]
pub struct MLMacroDef {
    pub span: Span,
    pub identifier: Ident,
    pub args: Option<MLMacroArgs>,
    pub defaults: Option<MLMacroDefDefaults>,
    pub contents: Vec<PASTNode>,
}

impl MLMacroDef {
    pub fn new(
        span: Span,
        identifier: Ident,
        args: Option<MLMacroArgs>,
        defaults: Option<MLMacroDefDefaults>,
        contents: Vec<PASTNode>,
    ) -> Self {
        Self {
            span,
            identifier,
            args,
            defaults,
            contents,
        }
    }
}

#[derive(Debug)]
pub struct MLMacroArgs {
    pub span: Span,
    pub required: u8,
    pub maximum: Option<NonZeroU8>,
}

impl MLMacroArgs {
    pub fn new(span: Span, required: u8, maximum: Option<NonZeroU8>) -> Self {
        Self {
            span,
            required,
            maximum,
        }
    }
}

#[derive(Debug)]
pub struct MLMacroDefDefaults {
    pub span: Span,
    pub values: Vec<BenignTokens>,
}

impl MLMacroDefDefaults {
    pub fn new(span: Span, values: Vec<BenignTokens>) -> Self {
        Self { span, values }
    }

    pub fn from_vec(values: Vec<BenignTokens>) -> Self {
        let mut span = Span::new(0, 0, 0);

        let first_span = values.first().unwrap().span;
        let last_span = values.last().unwrap().span;

        span.start = first_span.start;
        span.file = first_span.file;
        span.end = last_span.end;

        MLMacroDefDefaults { span, values }
    }
}

/// A PAST Node that represents a single line macro undefinition
///
/// Grammar:
///
/// ```sh,ignore,no_run
/// <SLMacroUndef> ::= .undef <ident>
///                |   .undef <ident> <SLMacroUndefArgs>
/// ```
///
#[derive(Debug)]
pub struct SLMacroUndef {
    pub span: Span,
    pub identifier: Ident,
    pub args: SLMacroUndefArgs,
}

impl SLMacroUndef {
    pub fn new(span: Span, identifier: Ident, args: SLMacroUndefArgs) -> Self {
        Self {
            span,
            identifier,
            args,
        }
    }
}

/// Represents a single line macro's number of arguments
///
/// ```sh,ignore,no_run
/// <SLMacroUndefArgs> ::= <number>
/// ```
///
#[derive(Debug)]
pub struct SLMacroUndefArgs {
    pub span: Span,
    pub num: u8,
}

impl SLMacroUndefArgs {
    pub fn new(span: Span, num: u8) -> Self {
        Self { span, num }
    }
}

/// A PAST Node that represents a multi line macro undefinition
///
/// Grammar:
///
/// ```sh,ignore,no_run
/// <MLMacroUndef> ::= .unmacro <ident>
///                |   .unmacro <ident> <MLMacroArgs>
/// ```
///
#[derive(Debug)]
pub struct MLMacroUndef {
    pub span: Span,
    pub identifier: Ident,
    pub args: MLMacroArgs,
}

impl MLMacroUndef {
    pub fn new(span: Span, identifier: Ident, args: MLMacroArgs) -> Self {
        Self {
            span,
            identifier,
            args,
        }
    }
}

/// A PAST node that represents a repeat directive
///
/// Grammar:
///
/// ```sh,ignore,no_run
/// <Repeat> ::= .rep <RepeatNumber>
/// ```
///
#[derive(Debug)]
pub struct Repeat {
    pub span: Span,
    pub number: RepeatNumber,
    pub contents: Vec<PASTNode>,
}

impl Repeat {
    pub fn new(span: Span, number: RepeatNumber, contents: Vec<PASTNode>) -> Self {
        Self {
            span,
            number,
            contents,
        }
    }
}

/// A PAST node that represents a repeat directive's number of repetitions
///
/// Grammar:
///
/// ```sh,ignore,no_run
/// <RepeatNumber> ::= <BenignTokens> | <MacroInvok>
/// ```
///
#[derive(Debug)]
pub struct RepeatNumber {
    pub span: Span,
    pub expression: Vec<PASTNode>,
}

impl RepeatNumber {
    pub fn new(span: Span, expression: Vec<PASTNode>) -> Self {
        Self { span, expression }
    }
}

#[derive(Debug)]
pub struct IfStatement {
    pub span: Span,
    pub clauses: Vec<IfClause>,
}

impl IfStatement {
    pub fn new(span: Span, clauses: Vec<IfClause>) -> Self {
        Self { span, clauses }
    }

    pub fn from_vec(clauses: Vec<IfClause>) -> Self {
        let mut span = Span::new(0, 0, 0);

        let first_span = clauses.first().unwrap().span;
        let last_span = clauses.last().unwrap().span;

        span.start = first_span.start;
        span.file = first_span.file;
        span.end = last_span.end;

        Self { span, clauses }
    }
}

#[derive(Debug)]
pub struct IfClause {
    pub span: Span,
    pub begin: IfClauseBegin,
    pub condition: IfCondition,
    pub contents: Vec<PASTNode>,
}

impl IfClause {
    pub fn new(
        span: Span,
        begin: IfClauseBegin,
        condition: IfCondition,
        contents: Vec<PASTNode>,
    ) -> Self {
        Self {
            span,
            begin,
            condition,
            contents,
        }
    }
}

/// This represents a single part like .if or .ifn
#[derive(Debug)]
pub struct IfClauseBegin {
    pub span: Span,
    pub inverse: bool,
}

impl IfClauseBegin {
    pub fn new(span: Span, inverse: bool) -> Self {
        Self { span, inverse }
    }
}

#[derive(Debug)]
pub enum IfCondition {
    Exp(IfExpCondition),
    Def(IfDefCondition),
}

#[derive(Debug)]
pub struct IfDefCondition {
    pub span: Span,
    pub identifier: Ident,
    pub args: Option<MLMacroArgs>,
}

impl IfDefCondition {
    pub fn new(span: Span, identifier: Ident, args: Option<MLMacroArgs>) -> Self {
        Self {
            span,
            identifier,
            args,
        }
    }
}

#[derive(Debug)]
pub struct IfExpCondition {
    pub span: Span,
    pub expression: Vec<PASTNode>,
}

impl IfExpCondition {
    pub fn new(span: Span, expression: Vec<PASTNode>) -> Self {
        Self { span, expression }
    }
}

#[derive(Debug)]
pub struct Include {
    pub span: Span,
    pub path: IncludePath,
}

impl Include {
    pub fn new(span: Span, path: IncludePath) -> Self {
        Self { span, path }
    }
}

#[derive(Debug)]
pub struct IncludePath {
    pub span: Span,
    pub expression: Vec<PASTNode>,
}

impl IncludePath {
    pub fn new(span: Span, expression: Vec<PASTNode>) -> Self {
        Self { span, expression }
    }
}
