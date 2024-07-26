use logos::Logos;

use crate::{errors::Span, session::Session};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Operator(Operator),
    Keyword(Keyword),
    Type(Type),
    Directive(Directive),
    Literal(Literal),
    Symbol(Symbol),

    /// Labels
    Label,
    InnerLabel,
    InnerLabelReference,

    MacroArgmentReference,

    Identifier,

    /// Delimiters
    Newline,
    Whitespace,
    Backslash,

    Comment,

    // Errors
    Error,
    JunkFloatError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operator {
    Minus,
    Plus,
    Compliment,
    Multiply,
    Divide,
    Mod,
    And,
    Or,
    Equals,
    NotEquals,
    Negate,
    GreaterThan,
    LessThan,
    GreaterEquals,
    LessEquals
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Symbol {
    LeftParen,
    RightParen,
    Comma,
    Hash,
    At
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Keyword {
    Section,
    Text,
    Data,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Literal {
    Integer,
    Float,
    Hex,
    Binary,
    True,
    False,
    String
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Type {
    I8,
    I16,
    I32,
    I32V,
    F64,
    F64V,
    S,
    SV,
    B,
    BV,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Directive {
    Extern,
    Global,
    Local,
    Line,
    Type,
    Value,
    Func,
    Preprocessor(PreprocessorDirective)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PreprocessorDirective {
    Define,
    Macro,
    EndMacro,
    Repeat,
    EndRepeat,
    Include,
    Undef,
    Unmacro,
    If(IfDirective)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IfDirective {
    If,
    IfNot,
    IfDef,
    IfNotDef,
    ElseIf,
    ElseIfNot,
    ElseIfDef,
    ElseIfNotDef,
    Else,
    EndIf
}

/// These are the raw tokens produced by Logos
#[derive(Debug, Clone, Copy, Logos, PartialEq, Eq)]
pub enum RawToken {
    #[error]
    Error,

    #[token(".section")]
    KeywordSection,

    #[token(".text")]
    KeywordText,

    #[token(".data")]
    KeywordData,

    #[token(".i8")]
    TypeI8,

    #[token(".i16")]
    TypeI16,

    #[token(".i32")]
    TypeI32,

    #[token(".i32v")]
    TypeI32V,

    #[token(".f64")]
    TypeF64,

    #[token(".f64v")]
    TypeF64V,

    #[token(".s")]
    TypeS,

    #[token(".sv")]
    TypeSV,

    #[token(".b")]
    TypeB,

    #[token(".bv")]
    TypeBV,

    #[token(".define")]
    DirectiveDefine,

    #[token(".macro")]
    DirectiveMacro,

    #[token(".endmacro")]
    DirectiveEndmacro,

    #[token(".rep")]
    DirectiveRepeat,

    #[token(".endrep")]
    DirectiveEndRepeat,

    #[token(".include")]
    DirectiveInclude,

    #[token(".extern")]
    DirectiveExtern,

    #[token(".global")]
    DirectiveGlobal,

    #[token(".local")]
    DirectiveLocal,

    #[token(".line")]
    DirectiveLine,

    #[token(".type")]
    DirectiveType,

    #[token(".value")]
    DirectiveValue,

    #[token(".undef")]
    DirectiveUndef,

    #[token(".unmacro")]
    DirectiveUnmacro,

    #[token(".func")]
    DirectiveFunc,

    #[token(".if")]
    DirectiveIf,

    #[token(".ifn")]
    DirectiveIfNot,

    #[token(".ifdef")]
    DirectiveIfDef,

    #[token(".ifndef")]
    DirectiveIfNotDef,

    #[token(".elif")]
    DirectiveElseIf,

    #[token(".elifn")]
    DirectiveElseIfNot,

    #[token(".elifdef")]
    DirectiveElseIfDef,

    #[token(".elifndef")]
    DirectiveElseIfNotDef,

    #[token(".else")]
    DirectiveElse,

    #[token(".endif")]
    DirectiveEndIf,

    #[regex(r"\.[_a-zA-Z][_a-zA-Z0-9]*")]
    InnerLabelReference,

    #[regex(r"\.[_a-zA-Z][_a-zA-Z0-9]*:")]
    InnerLabel,

    #[regex(r"[_a-zA-Z][_a-zA-Z0-9]*")]
    Identifier,

    #[regex(r"[_a-zA-Z][_a-zA-Z0-9]*:")]
    Label,

    #[regex(r"\&[0-9]+")]
    MacroArgmentReference,

    #[regex(r"[ \t\f]+")]
    Whitespace,

    #[regex("\r\n|\r|\n")]
    Newline,

    #[token("\\")]
    Backslash,

    #[regex(r"[0-9]+")]
    LiteralInteger,

    #[regex(r"[0-9]+\.[0-9]+")]
    LiteralFloat,

    #[regex(r"[0-9]+\.[0-9fe]*")]
    JunkFloatError,

    #[regex(r"0x[0-9a-fA-F][0-9a-fA-f_]*")]
    LiteralHex,

    #[regex(r"0b[01][01_]+")]
    LiteralBinary,

    #[token("true")]
    LiteralTrue,

    #[token("false")]
    LiteralFalse,

    #[regex("\"(?s:[^\"\\\\]|\\\\.)*\"")]
    LiteralString,

    #[token("-")]
    OperatorMinus,

    #[token("+")]
    OperatorPlus,

    #[token("~")]
    OperatorCompliment,

    #[token("*")]
    OperatorMultiply,

    #[token("/")]
    OperatorDivide,

    #[token("%")]
    OperatorMod,

    #[token("&&")]
    OperatorAnd,

    #[token("||")]
    OperatorOr,

    #[token("==")]
    OperatorEquals,

    #[token("!=")]
    OperatorNotEquals,

    #[token("!")]
    OperatorNegate,

    #[token(">")]
    OperatorGreaterThan,

    #[token("<")]
    OperatorLessThan,

    #[token(">=")]
    OperatorGreaterEquals,

    #[token("<=")]
    OperatorLessEquals,

    #[token("(")]
    SymbolLeftParen,

    #[token(")")]
    SymbolRightParen,

    #[token(",")]
    SymbolComma,

    #[token("#")]
    SymbolHash,

    #[token("@")]
    SymbolAt,

    #[regex(r";[^\n]*")]
    Comment,
}

/// Produced by the lexer, it is the smallest element that can be parsed, it contains the token's data and position in the source code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Token {
    /// The kind of token
    pub kind: TokenKind,

    /// The ID of the file this token belongs to
    pub file_id: u8,

    /// The index into the file's source that this token is
    pub source_index: u32,

    /// The length of the token in the source
    pub len: u16,
}

impl Token {
    /// Creates a new Span that envelopes just this token
    pub fn as_span(&self) -> Span {
        Span {
            start: self.source_index as usize,
            end: (self.source_index + (self.len as u32)) as usize,
            file: self.file_id as usize,
        }
    }

    /// Returns the string from the source code that represents this token. This could be parsed again and would return the same TokenKind as
    /// the original
    pub fn to_string(&self, session: &Session) -> String {
        let str_rep = match self.kind {
            TokenKind::Newline => "\n",
            TokenKind::Operator(Operator::Minus) => "-",
            TokenKind::Operator(Operator::Plus) => "+",
            TokenKind::Operator(Operator::Compliment) => "!",
            TokenKind::Operator(Operator::Multiply) => "*",
            TokenKind::Operator(Operator::Divide) => "/",
            TokenKind::Operator(Operator::Mod) => "%",
            TokenKind::Operator(Operator::And) => "&&",
            TokenKind::Operator(Operator::Or) => "||",
            TokenKind::Operator(Operator::Equals) => "==",
            TokenKind::Operator(Operator::NotEquals) => "!=",
            TokenKind::Operator(Operator::Negate) => "!",
            TokenKind::Operator(Operator::GreaterThan) => ">",
            TokenKind::Operator(Operator::LessThan) => "<",
            TokenKind::Operator(Operator::GreaterEquals) => ">=",
            TokenKind::Operator(Operator::LessEquals) => "<=",
            TokenKind::Symbol(Symbol::LeftParen) => "(",
            TokenKind::Symbol(Symbol::RightParen) => ")",
            TokenKind::Symbol(Symbol::Comma) => ",",
            TokenKind::Symbol(Symbol::Hash) => "#",
            TokenKind::Symbol(Symbol::At) => "@",
            TokenKind::Literal(Literal::True) => "true",
            TokenKind::Literal(Literal::False) => "false",
            TokenKind::Backslash => "\\",
            TokenKind::Keyword(Keyword::Section) => ".section",
            TokenKind::Keyword(Keyword::Text) => ".text",
            TokenKind::Keyword(Keyword::Data) => ".data",
            TokenKind::Type(Type::I8) => ".i8",
            TokenKind::Type(Type::I16) => ".i16",
            TokenKind::Type(Type::I32) => ".i32",
            TokenKind::Type(Type::I32V) => ".i32v",
            TokenKind::Type(Type::F64) => ".f64",
            TokenKind::Type(Type::F64V) => ".f64v",
            TokenKind::Type(Type::S) => ".s",
            TokenKind::Type(Type::SV) => ".sv",
            TokenKind::Type(Type::B) => ".b",
            TokenKind::Type(Type::BV) => ".bv",
            TokenKind::Directive(Directive::Extern) => ".extern",
            TokenKind::Directive(Directive::Global) => ".global",
            TokenKind::Directive(Directive::Local) => ".local",
            TokenKind::Directive(Directive::Line) => ".line",
            TokenKind::Directive(Directive::Type) => ".type",
            TokenKind::Directive(Directive::Value) => ".value",
            TokenKind::Directive(Directive::Func) => ".func",
            TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::Define)) => ".define",
            TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::Macro)) => ".macro",
            TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::EndMacro)) => ".endmacro",
            TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::Repeat)) => ".rep",
            TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::EndRepeat)) => ".endrep",
            TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::Include)) => ".include",
            TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::Undef)) => ".undef",
            TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::Unmacro)) => ".unmacro",
            TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::If(IfDirective::If))) => ".if",
            TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::If(IfDirective::IfNot))) => ".ifn",
            TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::If(IfDirective::IfDef))) => ".ifdef",
            TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::If(IfDirective::IfNotDef))) => ".ifndef",
            TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::If(IfDirective::ElseIf))) => ".elif",
            TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::If(IfDirective::ElseIfNot))) => ".elifn",
            TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::If(IfDirective::ElseIfDef))) => ".elifdef",
            TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::If(IfDirective::ElseIfNotDef))) => ".elifndef",
            TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::If(IfDirective::Else))) => ".else",
            TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::If(IfDirective::EndIf))) => ".endif",
            TokenKind::Label | TokenKind::InnerLabel | TokenKind::InnerLabelReference | TokenKind::MacroArgmentReference | TokenKind::Identifier | TokenKind::Whitespace | TokenKind::Comment | TokenKind::Error | TokenKind::JunkFloatError | TokenKind::Literal(_) => ""
        };

        if str_rep.is_empty() {
            let snippet = session.span_to_snippet(&self.as_span());
            return snippet.as_string();
        }

        return str_rep.to_string();
    }
}