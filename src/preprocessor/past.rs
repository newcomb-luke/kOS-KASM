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
pub struct PAST {
    nodes: Vec<PASTNode>,
}

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
    span: Span,
    hash: u64,
}

impl PartialEq for Ident {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

pub struct BenignTokens {
    span: Span,
    tokens: Vec<Token>,
}

pub struct SLMacroDef {
    span: Span,
    identifier: Ident,
    args: Option<SLMacroDefArgs>,
    contents: Option<SLMacroDefContents>,
}

pub struct SLMacroDefArgs {
    span: Span,
    args: Vec<Ident>,
}

pub struct SLMacroDefContents {
    span: Span,
    tokens: Vec<PASTNode>,
}

pub struct MacroInvok {
    span: Span,
    identifier: Ident,
    args: Option<MacroInvokArgs>,
}

pub struct MacroInvokArgs {
    span: Span,
    args: Vec<MacroInvokArg>,
}

pub struct MacroInvokArg {
    span: Span,
    tokens: Vec<PASTNode>,
}

pub struct MLMacroDef {
    span: Span,
    identifier: Ident,
    args: Option<MLMacroArgs>,
    defaults: Option<MLMacroDefDefaults>,
}

pub struct MLMacroArgs {
    span: Span,
    required: u8,
    maximum: Option<NonZeroU8>,
}

pub struct MLMacroDefDefaults {
    span: Span,
    values: Vec<MLMacroDefDefault>,
}

pub struct MLMacroDefDefault {
    span: Span,
    tokens: Vec<BenignTokens>,
}

pub struct SLMacroUndef {
    span: Span,
    identifier: Ident,
    args: Option<SLMacroUndefArgs>,
}

pub struct SLMacroUndefArgs {
    span: Span,
    num: u8,
}

pub struct MLMacroUndef {
    span: Span,
    identifier: Ident,
    args: Option<MLMacroArgs>,
}

pub struct Repeat {
    span: Span,
    number: RepeatNumber,
}

pub struct RepeatNumber {
    span: Span,
    expression: Vec<PASTNode>,
}

pub struct IfStatement {
    span: Span,
    clauses: Vec<IfClause>,
}

pub struct IfClause {
    span: Span,
    begin: IfClauseBegin,
    condition: IfCondition,
    contents: Vec<PASTNode>,
}

/// This represents a single part like .if or .ifn
pub struct IfClauseBegin {
    span: Span,
    inverse: bool,
}

pub enum IfCondition {
    Exp(IfExpCondition),
    Def(IfDefCondition),
}

pub struct IfDefCondition {
    span: Span,
    identifier: Ident,
    args: Option<MLMacroArgs>,
}

pub struct IfExpCondition {
    span: Span,
    expression: Vec<PASTNode>,
}

pub struct Include {
    span: Span,
    path: IncludePath,
}

pub struct IncludePath {
    span: Span,
    path: PathBuf,
}
