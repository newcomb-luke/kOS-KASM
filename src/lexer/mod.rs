pub mod token;
use logos::Logos;
use token::RawToken;
pub use token::*;

use crate::errors::Error;

use self::token::{Token, TokenKind};

pub struct Lexer<'a> {
    inner: logos::Lexer<'a, RawToken>,
    done: bool,
    len: usize,
    peeked: Option<Token>,
}

impl<'a> Lexer<'a> {
    /// Creates a new lexer
    pub fn new(source: &'a str) -> Lexer {
        Lexer {
            inner: RawToken::lexer(source),
            done: false,
            len: 0,
            peeked: None,
        }
    }

    // Lexes a single RawToken from the source input
    fn lex_raw(&mut self) -> Option<RawToken> {
        if self.done {
            None
        } else {
            if let Some(raw) = self.inner.next() {
                let raw_len = self.inner.slice().len();
                self.len += raw_len;

                Some(raw)
            } else {
                self.done = true;
                None
            }
        }
    }

    // Converts a RawToken into a Token
    fn raw_to_token(raw: RawToken, len: usize) -> Token {
        let kind = match raw {
            RawToken::OperatorMinus => TokenKind::OperatorMinus,
            RawToken::OperatorPlus => TokenKind::OperatorPlus,
            RawToken::OperatorCompliment => TokenKind::OperatorCompliment,
            RawToken::OperatorMultiply => TokenKind::OperatorMultiply,
            RawToken::OperatorDivide => TokenKind::OperatorDivide,
            RawToken::OperatorMod => TokenKind::OperatorMod,
            RawToken::OperatorAnd => TokenKind::OperatorAnd,
            RawToken::OperatorOr => TokenKind::OperatorOr,
            RawToken::OperatorEquals => TokenKind::OperatorEquals,
            RawToken::OperatorNotEquals => TokenKind::OperatorNotEquals,
            RawToken::OperatorGreaterThan => TokenKind::OperatorGreaterThan,
            RawToken::OperatorLessThan => TokenKind::OperatorLessThan,
            RawToken::OperatorGreaterEquals => TokenKind::OperatorGreaterEquals,
            RawToken::OperatorLessEquals => TokenKind::OperatorLessEquals,

            RawToken::KeywordSection => TokenKind::KeywordSection,
            RawToken::KeywordText => TokenKind::KeywordText,
            RawToken::KeywordData => TokenKind::KeywordData,

            RawToken::DirectiveDefine => TokenKind::DirectiveDefine,
            RawToken::DirectiveMacro => TokenKind::DirectiveMacro,
            RawToken::DirectiveRepeat => TokenKind::DirectiveRepeat,
            RawToken::DirectiveInclude => TokenKind::DirectiveInclude,
            RawToken::DirectiveExtern => TokenKind::DirectiveExtern,
            RawToken::DirectiveGlobal => TokenKind::DirectiveGlobal,
            RawToken::DirectiveLocal => TokenKind::DirectiveLocal,
            RawToken::DirectiveLine => TokenKind::DirectiveLine,
            RawToken::DirectiveUndef => TokenKind::DirectiveUndef,
            RawToken::DirectiveUnmacro => TokenKind::DirectiveUnmacro,
            RawToken::DirectiveFunc => TokenKind::DirectiveFunc,
            RawToken::DirectiveIf => TokenKind::DirectiveIf,
            RawToken::DirectiveIfNot => TokenKind::DirectiveIfNot,
            RawToken::DirectiveIfDef => TokenKind::DirectiveIfDef,
            RawToken::DirectiveIfNotDef => TokenKind::DirectiveIfNotDef,
            RawToken::DirectiveElseIf => TokenKind::DirectiveElseIf,
            RawToken::DirectiveElseIfNot => TokenKind::DirectiveElseIfNot,
            RawToken::DirectiveElseIfDef => TokenKind::DirectiveElseIfDef,
            RawToken::DirectiveElseIfNotDef => TokenKind::DirectiveElseIfNotDef,
            RawToken::DirectiveElse => TokenKind::DirectiveElse,
            RawToken::DirectiveEndIf => TokenKind::DirectiveEndIf,

            RawToken::Label => TokenKind::Label,
            RawToken::InnerLabel => TokenKind::InnerLabel,

            RawToken::InnerLabelReference => TokenKind::InnerLabelReference,

            RawToken::Identifier => TokenKind::Identifier,

            RawToken::LiteralInteger => TokenKind::LiteralInteger,
            RawToken::LiteralFloat => TokenKind::LiteralFloat,
            RawToken::LiteralHex => TokenKind::LiteralHex,
            RawToken::LiteralBinary => TokenKind::LiteralBinary,
            RawToken::LiteralTrue => TokenKind::LiteralTrue,
            RawToken::LiteralFalse => TokenKind::LiteralFalse,
            RawToken::LiteralString => TokenKind::LiteralString,

            RawToken::Newline => TokenKind::Newline,
            RawToken::Whitespace => TokenKind::Whitespace,
            RawToken::Backslash => TokenKind::Backslash,

            RawToken::SymbolLeftParen => TokenKind::SymbolLeftParen,
            RawToken::SymbolRightParen => TokenKind::SymbolRightParen,
            RawToken::SymbolComma => TokenKind::SymbolComma,
            RawToken::SymbolHash => TokenKind::SymbolHash,
            RawToken::SymbolAt => TokenKind::SymbolAt,
            RawToken::SymbolAnd => TokenKind::SymbolAnd,

            RawToken::Comment => TokenKind::Comment,

            RawToken::Error => TokenKind::Error,
            RawToken::JunkFloatError => TokenKind::JunkFloatError,
        };

        Token {
            kind,
            len: len as u32,
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(peeked) = self.peeked {
            let peeked_token = peeked;
            self.peeked = None;
            Some(peeked_token)
        } else {
            let raw_token = self.lex_raw()?;
            Some(Self::raw_to_token(raw_token, self.inner.slice().len()))
        }
    }
}

/// Tokenize a string of KASM into an iterator of tokens.
pub fn tokenize<'a>(source: &'a str) -> impl Iterator<Item = Token> + 'a {
    Lexer::new(source)
}

/// Checks the token iterator for errors, and if one appears, returns an error
pub fn check_errors<'a>(tokens: &Vec<Token>) -> Result<(), Vec<Error>> {
    let mut errors = Vec::new();

    for (token_index, token) in tokens.iter().enumerate() {
        if token.kind == TokenKind::Error {
            errors.push(Error::new(
                crate::errors::ErrorKind::TokenParse,
                token_index as u32,
            ));
        } else if token.kind == TokenKind::JunkFloatError {
            errors.push(Error::new(
                crate::errors::ErrorKind::JunkFloat,
                token_index as u32,
            ));
        }
    }

    if errors.len() > 0 {
        Err(errors)
    } else {
        Ok(())
    }
}
