use std::num::NonZeroU8;
use std::path::PathBuf;

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
    span: Span,
    tokens: Vec<Token>,
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
/// ```
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
/// ```
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
/// ```
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

#[derive(Debug)]
pub struct MacroInvokArg {
    span: Span,
    tokens: Vec<PASTNode>,
}

#[derive(Debug)]
pub struct MLMacroDef {
    span: Span,
    identifier: Ident,
    args: Option<MLMacroArgs>,
    defaults: Option<MLMacroDefDefaults>,
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
    span: Span,
    values: Vec<MLMacroDefDefault>,
}

#[derive(Debug)]
pub struct MLMacroDefDefault {
    span: Span,
    tokens: Vec<BenignTokens>,
}

/// A PAST Node that represents a single line macro undefinition
///
/// Grammar:
///
/// ```
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
/// ```
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

#[derive(Debug)]
pub struct Repeat {
    span: Span,
    number: RepeatNumber,
}

#[derive(Debug)]
pub struct RepeatNumber {
    span: Span,
    expression: Vec<PASTNode>,
}

#[derive(Debug)]
pub struct IfStatement {
    span: Span,
    clauses: Vec<IfClause>,
}

#[derive(Debug)]
pub struct IfClause {
    span: Span,
    begin: IfClauseBegin,
    condition: IfCondition,
    contents: Vec<PASTNode>,
}

/// This represents a single part like .if or .ifn
#[derive(Debug)]
pub struct IfClauseBegin {
    span: Span,
    inverse: bool,
}

#[derive(Debug)]
pub enum IfCondition {
    Exp(IfExpCondition),
    Def(IfDefCondition),
}

#[derive(Debug)]
pub struct IfDefCondition {
    span: Span,
    identifier: Ident,
    args: Option<MLMacroArgs>,
}

#[derive(Debug)]
pub struct IfExpCondition {
    span: Span,
    expression: Vec<PASTNode>,
}

#[derive(Debug)]
pub struct Include {
    span: Span,
    path: IncludePath,
}

#[derive(Debug)]
pub struct IncludePath {
    span: Span,
    path: PathBuf,
}
