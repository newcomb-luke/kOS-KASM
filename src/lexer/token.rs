use logos::Logos;

use crate::errors::{AssemblyError, ErrorKind, ErrorManager, KASMResult, SourceFile};

#[repr(u8)]
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
    OperatorNegate,
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
    DirectiveEndmacro,
    DirectiveRepeat,
    DirectiveInclude,
    DirectiveExtern,
    DirectiveGlobal,
    DirectiveLocal,
    DirectiveLine,
    DirectiveType,
    DirectiveValue,
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

    /// Labels
    Label,
    InnerLabel,

    InnerLabelReference,

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

    #[token(".endmacro")]
    DirectiveEndmacro,

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

    #[regex(r"\.[_a-zA-Z][_a-zA-Z0-9]+")]
    InnerLabelReference,

    #[regex(r"\.[_a-zA-Z][_a-zA-Z0-9]+:")]
    InnerLabel,

    #[regex(r"[_a-zA-Z][_a-zA-Z0-9]?")]
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

    #[token("&")]
    SymbolAnd,

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

impl<'a> Token {
    /// Tries to find the slice of the source code that corresponds to this token
    pub fn slice(&self, source_files: &'a Vec<SourceFile>) -> Option<&'a str> {
        let file = source_files.get(self.file_id as usize)?;

        let start = self.source_index as usize;
        let end = start + self.len as usize;

        file.source().get(start..end)
    }
}

/// An "iterator" that will iterate over Tokens, but with extra features a normal iterator does not
/// have.
pub struct TokenIter {
    tokens: Vec<Token>,
    position: usize,
}

impl TokenIter {
    /// Creates a new token iterator that will iterate over the provided token vector
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    /// Gets the next Token, if there is one. If not, returns None
    pub fn next(&mut self) -> Option<&Token> {
        // Get the element
        let token = self.tokens.get(self.position);

        // Advance the iterator
        self.position += 1;

        token
    }

    /// Gets the next Token but does not advance the iterator, if there is one. It not, returns
    /// None
    pub fn peek(&self) -> Option<&Token> {
        // Get the element
        self.tokens.get(self.position)
    }

    /// Gets the next Token but does not advance the iterator, if there is one. If not, it returns
    /// an Err() with the provided error value
    pub fn peek_or<E>(&self, err: E) -> Result<&Token, E> {
        self.tokens.get(self.position).ok_or(err)
    }

    /// Gets the Token at a specified index into the iterator, if there is one. If not, returns
    /// None
    pub fn get(&self, index: usize) -> Option<&Token> {
        // Get the element
        self.tokens.get(index)
    }

    /// Gets the Token that was previously returned from a call to next(), if there is one. If not,
    /// returns None
    pub fn previous(&self) -> Option<&Token> {
        // If we won't be trying to access the -1th element...
        if self.position > 0 {
            // Get the previous element
            self.tokens.get(self.position - 1)
        } else {
            None
        }
    }

    /// Returns the current position of the iterator
    pub fn position(&self) -> usize {
        self.position
    }

    /// Inserts another vector of Tokens into the iterator into the current position
    pub fn insert(&mut self, tokens: Vec<Token>) {
        for token in tokens {
            self.tokens.insert(self.position, token);
        }
    }
}

impl From<TokenIter> for Vec<Token> {
    fn from(iter: TokenIter) -> Self {
        iter.tokens
    }
}

#[test]
fn test_peek() {
    let tokens = vec![
        Token {
            kind: TokenKind::Backslash,
            file_id: 0,
            source_index: 0,
            len: 1,
        },
        Token {
            kind: TokenKind::Newline,
            file_id: 0,
            source_index: 1,
            len: 1,
        },
        Token {
            kind: TokenKind::Identifier,
            file_id: 0,
            source_index: 2,
            len: 5,
        },
    ];

    let mut token_iter = TokenIter::new(tokens);

    assert_eq!(token_iter.peek().unwrap().kind, TokenKind::Backslash);
    assert_eq!(token_iter.peek().unwrap().kind, TokenKind::Backslash);
    assert_eq!(token_iter.next().unwrap().kind, TokenKind::Backslash);
    assert_eq!(token_iter.peek().unwrap().kind, TokenKind::Newline);
    assert_eq!(token_iter.next().unwrap().kind, TokenKind::Newline);
    assert_eq!(token_iter.peek().unwrap().kind, TokenKind::Identifier);
}

/// Consumes a single token from the TokenIter. If the token is not present, then it returns the
/// specified error to the ErrorManager
///
/// Behaves just like peek_token, but this consumes the token
pub fn consume_token(
    token_iter: &mut TokenIter,
    errors: &mut ErrorManager,
    missing: ErrorKind,
) -> KASMResult<Token> {
    // Check if there is a token
    if let Some(token) = token_iter.next() {
        Ok(*token)
    }
    // If not, register the error
    else {
        errors.add_assembly(AssemblyError::new(missing, *token_iter.previous().unwrap()));
        Err(())
    }
}

/// Peeks a single token from the TokenIter. If the token is not present, then it returns the
/// specified error to the ErrorManager
///
/// Behaves just like consume_token, but this just peeks the iterator
pub fn peek_token<'a>(
    token_iter: &'a mut TokenIter,
    errors: &mut ErrorManager,
    missing: ErrorKind,
) -> KASMResult<&'a Token> {
    // Check if there is a token
    if let Some(token) = token_iter.peek() {
        Ok(token)
    }
    // If not, register the error
    else {
        errors.add_assembly(AssemblyError::new(missing, *token_iter.previous().unwrap()));
        Err(())
    }
}

/// Consumes a single token from the TokenIter. If the token is not present, then it returns the
/// specified "missing" error kind to the ErrorManager. If the token is present, but the token kind
/// is incorrect, then the specified "incorrect" error kind is returned to the ErrorManager. If the
/// token is both present, and of the correct kind, it is returned as Ok(&Token)
pub fn parse_token(
    token_iter: &mut TokenIter,
    errors: &mut ErrorManager,
    kind: TokenKind,
    incorrect: ErrorKind,
    missing: ErrorKind,
) -> KASMResult<Token> {
    // Check for the token
    let token = consume_token(token_iter, errors, missing)?;

    if token.kind == kind {
        Ok(token)
    } else {
        errors.add_assembly(AssemblyError::new(incorrect, token));
        Err(())
    }
}

/// Peeks a token from the TokenIter. If the token is not present, then it registers the specified
/// error with the ErrorManager. If the token is present, then it checks to see if it is of the
/// kind specified. If it is, then it returns Ok(true), if not, it returns Ok(false)
pub fn test_next_is<'a>(
    token_iter: &'a mut TokenIter,
    errors: &mut ErrorManager,
    kind: TokenKind,
    missing: ErrorKind,
) -> KASMResult<bool> {
    // Check for the token
    let token = peek_token(token_iter, errors, missing)?;

    Ok(token.kind == kind)
}

#[test]
fn test_insert() {
    let first_tokens = vec![
        Token {
            kind: TokenKind::Backslash,
            file_id: 0,
            source_index: 0,
            len: 1,
        },
        Token {
            kind: TokenKind::Newline,
            file_id: 0,
            source_index: 1,
            len: 1,
        },
    ];

    let second_tokens = vec![Token {
        kind: TokenKind::DirectiveIf,
        file_id: 1,
        source_index: 0,
        len: 3,
    }];

    let mut token_iter = TokenIter::new(first_tokens);

    assert_eq!(token_iter.next().unwrap().kind, TokenKind::Backslash);

    token_iter.insert(second_tokens);

    assert_eq!(token_iter.next().unwrap().kind, TokenKind::DirectiveIf);

    assert_eq!(token_iter.next().unwrap().kind, TokenKind::Newline);

    assert_eq!(token_iter.next(), None);
}
