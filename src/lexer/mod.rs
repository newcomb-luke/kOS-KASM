mod token;
use logos::Logos;
use token::RawToken;
pub use token::*;

use crate::session::Session;

pub struct Lexer<'a> {
    inner: logos::Lexer<'a, RawToken>,
    done: bool,
    current_index: usize,
}

impl<'a> Lexer<'a> {
    /// Creates a new lexer
    pub fn new(source: &'a str) -> Lexer {
        Lexer {
            inner: RawToken::lexer(source),
            done: false,
            current_index: 0,
        }
    }

    // Lexes a single RawToken from the source input
    fn lex_raw(&mut self) -> Option<RawToken> {
        if self.done {
            None
        } else {
            if let Some(raw) = self.inner.next() {
                Some(raw)
            } else {
                self.done = true;
                None
            }
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
            file_id: 0,
            source_index,
            len: len as u16,
        }
    }
}
