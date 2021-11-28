#![allow(clippy::result_unit_err)]

mod token;
use logos::Logos;
use token::RawToken;
pub use token::*;

use crate::session::Session;

pub struct Lexer<'a, 'b> {
    inner: logos::Lexer<'a, RawToken>,
    done: bool,
    current_index: usize,
    session: &'b Session,
    file_id: u8,
}

impl<'a, 'b> Lexer<'a, 'b> {
    /// Creates a new lexer
    pub fn new(source: &'a str, file_id: u8, session: &'b Session) -> Lexer<'a, 'b> {
        Lexer {
            inner: RawToken::lexer(source),
            done: false,
            current_index: 0,
            session,
            file_id,
        }
    }

    /// This lexes the given input using the lexer. This returns a Result that contains a tuple of
    /// a token Vec and a Session that was provided when this lexer was created. This consumes the
    /// lexer
    pub fn lex(mut self) -> Result<Vec<Token>, ()> {
        let mut tokens = Vec::new();
        let mut fail = false;

        // Get all of the tokens, one by one
        while let Some(token) = self.next() {
            // Check if this token is an error token
            if token.kind == TokenKind::Error {
                self.session
                    .struct_span_error(token.as_span(), "unknown token".to_string())
                    .emit();

                fail = true;
            } else if token.kind == TokenKind::JunkFloatError {
                self.session
                    .struct_span_error(
                        token.as_span(),
                        "invalid floating point literal".to_string(),
                    )
                    .emit();

                fail = true;
            }

            tokens.push(token);
        }

        if fail {
            Err(())
        } else {
            Ok(tokens)
        }
    }

    // Properly gets the next token
    fn next(&mut self) -> Option<Token> {
        let raw_token = self.lex_raw()?;
        Some(self.raw_to_token(raw_token, self.inner.slice().len() as u16))
    }

    // Lexes a single RawToken from the source input
    fn lex_raw(&mut self) -> Option<RawToken> {
        if self.done {
            None
        } else if let Some(raw) = self.inner.next() {
            Some(raw)
        } else {
            self.done = true;
            None
        }
    }

    // Converts a RawToken into a Token
    fn raw_to_token(&mut self, raw: RawToken, len: u16) -> Token {
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
            RawToken::OperatorNegate => TokenKind::OperatorNegate,
            RawToken::OperatorGreaterThan => TokenKind::OperatorGreaterThan,
            RawToken::OperatorLessThan => TokenKind::OperatorLessThan,
            RawToken::OperatorGreaterEquals => TokenKind::OperatorGreaterEquals,
            RawToken::OperatorLessEquals => TokenKind::OperatorLessEquals,

            RawToken::KeywordSection => TokenKind::KeywordSection,
            RawToken::KeywordText => TokenKind::KeywordText,
            RawToken::KeywordData => TokenKind::KeywordData,

            RawToken::DirectiveDefine => TokenKind::DirectiveDefine,
            RawToken::DirectiveMacro => TokenKind::DirectiveMacro,
            RawToken::DirectiveEndmacro => TokenKind::DirectiveEndmacro,
            RawToken::DirectiveRepeat => TokenKind::DirectiveRepeat,
            RawToken::DirectiveEndRepeat => TokenKind::DirectiveEndRepeat,
            RawToken::DirectiveInclude => TokenKind::DirectiveInclude,
            RawToken::DirectiveExtern => TokenKind::DirectiveExtern,
            RawToken::DirectiveGlobal => TokenKind::DirectiveGlobal,
            RawToken::DirectiveLocal => TokenKind::DirectiveLocal,
            RawToken::DirectiveLine => TokenKind::DirectiveLine,
            RawToken::DirectiveType => TokenKind::DirectiveType,
            RawToken::DirectiveValue => TokenKind::DirectiveValue,
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

        let source_index = self.current_index as u32;

        self.current_index += len as usize;

        Token {
            kind,
            file_id: self.file_id,
            source_index,
            len: len as u16,
        }
    }
}

/// Replace comments and line continuations with whitespace tokens
pub fn phase0(tokens: &mut Vec<Token>, session: &Session) -> Result<(), ()> {
    let mut last_was_backslash = false;
    let mut fail = false;

    // Loop through all of the tokens
    for token in tokens.iter_mut() {
        // If the last token was a backslash (line continue)
        if last_was_backslash {
            // If it was a newline as expected, then replace it with whitespace and reset
            if token.kind == TokenKind::Newline {
                token.kind = TokenKind::Whitespace;
                last_was_backslash = false;
            }
            // If it was whitespace that is fine
            else if token.kind != TokenKind::Whitespace {
                // If it wasn't though, that is an error
                session
                    .struct_span_error(
                        token.as_span(),
                        "unexpected token after backslash".to_string(),
                    )
                    .emit();

                // We should try to keep going for errors' sake, so mark this as okay
                last_was_backslash = false;

                fail = true;
            }
        } else {
            match token.kind {
                // If it is a comment, replace it with whitespafce
                TokenKind::Comment => {
                    token.kind = TokenKind::Whitespace;
                }
                // If it is a backslash, replace it and prepare next iteration
                TokenKind::Backslash => {
                    token.kind = TokenKind::Whitespace;
                    last_was_backslash = true;
                }
                _ => {}
            }
        }
    }

    if fail {
        Err(())
    } else {
        Ok(())
    }
}
