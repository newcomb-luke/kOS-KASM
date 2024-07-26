use std::num::NonZeroU8;

use crate::errors::Span;
use crate::lexer::Token;

/// PAST stands for Preprocessor Abstract Syntax Tree
///
/// Basically, in KASM the preprocessor is treated as a tiny programming language, and is first
/// parsed, then "generated" which means that it generates the rest of the code that will be used
/// in KASM's subsequent operation.
///

#[derive(Debug, Clone)]
pub enum PASTNode {
    InertTokens(InertTokens),
    SLMacroDef(SLMacroDef),
    MacroInvocation(MacroInvocation),
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
            PASTNode::InertTokens(inert_tokens) => inert_tokens.span.end,
            PASTNode::SLMacroDef(sl_macro_def) => sl_macro_def.span.end,
            PASTNode::MacroInvocation(macro_invocation) => macro_invocation.span.end,
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

#[derive(Debug, Clone)]
pub struct InertTokens {
    pub span: Span,
    pub tokens: Vec<Token>,
}

impl InertTokens {
    /// Creates a new InertTokens struct using the tokens provided
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct SLMacroDefContents {
    pub span: Span,
    pub contents: Vec<PASTNode>,
}

impl SLMacroDefContents {
    pub fn new(span: Span, contents: Vec<PASTNode>) -> Self {
        Self { span, contents }
    }
}

#[derive(Debug, Clone)]
pub struct MacroInvocation {
    pub span: Span,
    pub identifier: Ident,
    pub args: Option<MacroInvocationArgs>,
}

impl MacroInvocation {
    pub fn new(span: Span, identifier: Ident, args: Option<MacroInvocationArgs>) -> Self {
        Self {
            span,
            identifier,
            args,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MacroInvocationArgs {
    pub span: Span,
    pub args: Vec<MacroInvocationArg>,
}

impl MacroInvocationArgs {
    pub fn new(span: Span, args: Vec<MacroInvocationArg>) -> Self {
        Self { span, args }
    }

    pub fn from_vec(args: Vec<MacroInvocationArg>) -> Self {
        let mut span = Span::new(0, 0, 0);

        let first_span = args.first().unwrap().span;
        let last_span = args.last().unwrap().span;

        span.start = first_span.start;
        span.file = first_span.file;
        span.end = last_span.end;

        MacroInvocationArgs { span, args }
    }
}

#[derive(Debug, Clone)]
pub struct MacroInvocationArg {
    pub span: Span,
    pub contents: Vec<PASTNode>,
}

impl MacroInvocationArg {
    pub fn new(span: Span, contents: Vec<PASTNode>) -> Self {
        Self { span, contents }
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

    /// Returns either the number of required args, or the maximum if it is Some()
    pub fn overall_maximum(&self) -> u8 {
        self.maximum.map(|v| v.get()).unwrap_or(self.required)
    }
}

#[derive(Debug, Clone)]
pub struct MLMacroDefDefaults {
    pub span: Span,
    pub values: Vec<InertTokens>,
}

impl MLMacroDefDefaults {
    pub fn new(span: Span, values: Vec<InertTokens>) -> Self {
        Self { span, values }
    }

    pub fn from_vec(values: Vec<InertTokens>) -> Self {
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct RepeatNumber {
    pub span: Span,
    pub expression: Vec<PASTNode>,
}

impl RepeatNumber {
    pub fn new(span: Span, expression: Vec<PASTNode>) -> Self {
        Self { span, expression }
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct IfClauseBegin {
    pub span: Span,
    pub inverse: bool,
}

impl IfClauseBegin {
    pub fn new(span: Span, inverse: bool) -> Self {
        Self { span, inverse }
    }
}

#[derive(Debug, Clone)]
pub enum IfCondition {
    Exp(IfExpCondition),
    Def(IfDefCondition),
    Else,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct IfExpCondition {
    pub span: Span,
    pub expression: Vec<PASTNode>,
}

impl IfExpCondition {
    pub fn new(span: Span, expression: Vec<PASTNode>) -> Self {
        Self { span, expression }
    }
}

#[derive(Debug, Clone)]
pub struct Include {
    pub span: Span,
    pub path: IncludePath,
}

impl Include {
    pub fn new(span: Span, path: IncludePath) -> Self {
        Self { span, path }
    }
}

#[derive(Debug, Clone)]
pub enum IncludePath {
    Literal(Span, String),
    MacroInvocation(MacroInvocation)
}