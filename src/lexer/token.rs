use logos::Logos;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    /// Operators
    OperatorMinus,
    OperatorPlus,
    OperatorCompliment,
    OperatorMultiply,
    OperatorDivide,
    OperatorMod,
    OperatorAnd,
    OperatorOr,
    OperatorEquals,
    OperatorNotEquals,
    OperatorGreaterThan,
    OperatorLessThan,
    OperatorGreaterEquals,
    OperatorLessEquals,

    /// Keywords
    KeywordSection,
    KeywordText,
    KeywordData,

    /// Directives
    DirectiveDefine,
    DirectiveMacro,
    DirectiveRepeat,
    DirectiveInclude,
    DirectiveExtern,
    DirectiveGlobal,
    DirectiveLocal,
    DirectiveLine,
    DirectiveUndef,
    DirectiveUnmacro,
    DirectiveFunc,
    DirectiveIf,
    DirectiveIfNot,
    DirectiveIfDef,
    DirectiveIfNotDef,
    DirectiveElseIf,
    DirectiveElseIfNot,
    DirectiveElseIfDef,
    DirectiveElseIfNotDef,
    DirectiveElse,
    DirectiveEndIf,
    Directive,

    /// Labels
    Label,
    InnerLabel,

    Identifier,

    /// Literals
    LiteralInteger,
    LiteralFloat,
    LiteralHex,
    LiteralBinary,
    LiteralTrue,
    LiteralFalse,
    LiteralString,

    /// Delimiters
    Newline,
    Whitespace,
    Backslash,

    /// Symbols
    SymbolLeftParen,
    SymbolRightParen,
    SymbolComma,
    SymbolHash,
    SymbolAt,
    SymbolAnd,

    Comment,

    // Errors
    Error,
    JunkFloatError,
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

    #[token(".define")]
    DirectiveDefine,

    #[token(".macro")]
    DirectiveMacro,

    #[token(".rep")]
    DirectiveRepeat,

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

    #[token("ifndef")]
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

    #[regex(r"\.[_a-zA-Z][_a-zA-Z0-9]+")]
    Directive,

    #[regex(r"\.[_a-zA-Z][_a-zA-Z0-9]+:")]
    InnerLabel,

    #[regex(r"[_a-zA-Z][_a-zA-Z0-9]+")]
    Identifier,

    #[regex(r"[_a-zA-Z][_a-zA-Z0-9]+:")]
    Label,

    #[regex(r"[ \t\f]+")]
    Whitespace,

    #[token("\n")]
    Newline,

    #[token("\\")]
    Backslash,

    #[regex(r"[0-9]+")]
    LiteralInteger,

    #[regex(r"[0-9]+\.[0-9]+")]
    LiteralFloat,

    #[regex(r"[0-9]+\.[\D&&\S]+")]
    JunkFloatError,

    #[regex(r"0x[0-9a-fA-F][0-9a-fA-f_]+")]
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

    #[token("&")]
    SymbolAnd,

    #[regex(r";[^\n]*")]
    Comment,
}

// Produced by the lexer, it is the smallest element that can be parsed, it contains the token's data and position in the source code
/// A token of KASM source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Token {
    /// The kind of topen
    pub kind: TokenKind,

    /// The token's length
    pub len: u32,
}
