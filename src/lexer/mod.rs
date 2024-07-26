#![allow(clippy::result_unit_err)]

mod token;
use logos::Logos;
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
            RawToken::OperatorMinus => TokenKind::Operator(Operator::Minus),
            RawToken::OperatorPlus => TokenKind::Operator(Operator::Plus),
            RawToken::OperatorCompliment => TokenKind::Operator(Operator::Compliment),
            RawToken::OperatorMultiply => TokenKind::Operator(Operator::Multiply),
            RawToken::OperatorDivide => TokenKind::Operator(Operator::Divide),
            RawToken::OperatorMod => TokenKind::Operator(Operator::Mod),
            RawToken::OperatorAnd => TokenKind::Operator(Operator::And),
            RawToken::OperatorOr => TokenKind::Operator(Operator::Or),
            RawToken::OperatorEquals => TokenKind::Operator(Operator::Equals),
            RawToken::OperatorNotEquals => TokenKind::Operator(Operator::NotEquals),
            RawToken::OperatorNegate => TokenKind::Operator(Operator::Negate),
            RawToken::OperatorGreaterThan => TokenKind::Operator(Operator::GreaterThan),
            RawToken::OperatorLessThan => TokenKind::Operator(Operator::LessThan),
            RawToken::OperatorGreaterEquals => TokenKind::Operator(Operator::GreaterEquals),
            RawToken::OperatorLessEquals => TokenKind::Operator(Operator::LessEquals),

            RawToken::KeywordSection => TokenKind::Keyword(Keyword::Section),
            RawToken::KeywordText => TokenKind::Keyword(Keyword::Text),
            RawToken::KeywordData => TokenKind::Keyword(Keyword::Data),

            RawToken::TypeI8 => TokenKind::Type(Type::I8),
            RawToken::TypeI16 => TokenKind::Type(Type::I16),
            RawToken::TypeI32 => TokenKind::Type(Type::I32),
            RawToken::TypeI32V => TokenKind::Type(Type::I32V),
            RawToken::TypeF64 => TokenKind::Type(Type::F64),
            RawToken::TypeF64V => TokenKind::Type(Type::F64V),
            RawToken::TypeS => TokenKind::Type(Type::S),
            RawToken::TypeSV => TokenKind::Type(Type::SV),
            RawToken::TypeB => TokenKind::Type(Type::B),
            RawToken::TypeBV => TokenKind::Type(Type::BV),

            RawToken::DirectiveDefine => TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::Define)),
            RawToken::DirectiveMacro => TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::Macro)),
            RawToken::DirectiveEndmacro => TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::EndMacro)),
            RawToken::DirectiveRepeat => TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::Repeat)),
            RawToken::DirectiveEndRepeat => TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::EndRepeat)),
            RawToken::DirectiveInclude => TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::Include)),
            RawToken::DirectiveExtern => TokenKind::Directive(Directive::Extern),
            RawToken::DirectiveGlobal => TokenKind::Directive(Directive::Global),
            RawToken::DirectiveLocal => TokenKind::Directive(Directive::Local),
            RawToken::DirectiveLine => TokenKind::Directive(Directive::Line),
            RawToken::DirectiveType => TokenKind::Directive(Directive::Type),
            RawToken::DirectiveValue => TokenKind::Directive(Directive::Value),
            RawToken::DirectiveUndef => TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::Undef)),
            RawToken::DirectiveUnmacro => TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::Unmacro)),
            RawToken::DirectiveFunc => TokenKind::Directive(Directive::Func),
            RawToken::DirectiveIf => TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::If(IfDirective::If))),
            RawToken::DirectiveIfNot => TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::If(IfDirective::IfNot))),
            RawToken::DirectiveIfDef => TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::If(IfDirective::IfDef))),
            RawToken::DirectiveIfNotDef => TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::If(IfDirective::IfNotDef))),
            RawToken::DirectiveElseIf => TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::If(IfDirective::ElseIf))),
            RawToken::DirectiveElseIfNot => TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::If(IfDirective::ElseIfNot))),
            RawToken::DirectiveElseIfDef => TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::If(IfDirective::ElseIfDef))),
            RawToken::DirectiveElseIfNotDef => TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::If(IfDirective::ElseIfNotDef))),
            RawToken::DirectiveElse => TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::If(IfDirective::Else))),
            RawToken::DirectiveEndIf => TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::If(IfDirective::EndIf))),

            RawToken::Label => TokenKind::Label,
            RawToken::InnerLabel => TokenKind::InnerLabel,
            RawToken::InnerLabelReference => TokenKind::InnerLabelReference,

            RawToken::MacroArgmentReference => TokenKind::MacroArgmentReference,

            RawToken::Identifier => TokenKind::Identifier,

            RawToken::LiteralInteger => TokenKind::Literal(Literal::Integer),
            RawToken::LiteralFloat => TokenKind::Literal(Literal::Float),
            RawToken::LiteralHex => TokenKind::Literal(Literal::Hex),
            RawToken::LiteralBinary => TokenKind::Literal(Literal::Binary),
            RawToken::LiteralTrue => TokenKind::Literal(Literal::True),
            RawToken::LiteralFalse => TokenKind::Literal(Literal::False),
            RawToken::LiteralString => TokenKind::Literal(Literal::String),

            RawToken::Newline => TokenKind::Newline,
            RawToken::Whitespace => TokenKind::Whitespace,
            RawToken::Backslash => TokenKind::Backslash,

            RawToken::SymbolLeftParen => TokenKind::Symbol(Symbol::LeftParen),
            RawToken::SymbolRightParen => TokenKind::Symbol(Symbol::RightParen),
            RawToken::SymbolComma => TokenKind::Symbol(Symbol::Comma),
            RawToken::SymbolHash => TokenKind::Symbol(Symbol::Hash),
            RawToken::SymbolAt => TokenKind::Symbol(Symbol::At),

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
