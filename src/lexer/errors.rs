use std::{error::Error, fmt::Display, fmt::Formatter};

pub type LexResult<T> = Result<T, LexError>;

#[derive(Debug)]
pub enum LexError {
    TokenParseError(usize),
}

pub trait TokenIndex {
    fn token_index(&self) -> usize;
}

pub trait Emittable: IndexEmittable {
    fn emit(&self, file_context: &FileContext, tokens: &Vec<Token>) -> std::io::Result<()> {
        let mut index = 0;
        let mut error_token = None;

        for (iter_index, token) in tokens.iter().enumerate() {
            if iter_index == self.token_index() {
                error_token = Some(token);
                break;
            }

            index += token.len;
        }

        if let Some(token) = error_token {
            self.emit_index(file_context, index, token.clone())
        } else {
            Console::emit(
                crate::output::console::Level::Bug,
                file_context,
                "Internal error in assembler",
                "",
                0,
                0,
            )
        }
    }
}

pub trait IndexEmittable: TokenIndex {
    fn emit_index(
        &self,
        file_context: &FileContext,
        index: u32,
        token: Token,
    ) -> std::io::Result<()>;
}

impl TokenIndex for Phase0Error {
    fn token_index(&self) -> usize {
        match self {
            Phase0Error::JunkAfterBackslashError(token_index) => *token_index,
        }
    }
}

impl IndexEmittable for Phase0Error {
    fn emit_index(
        &self,
        file_context: &FileContext,
        index: u32,
        token: Token,
    ) -> std::io::Result<()> {
        let (prefix, message) = match self {
            Phase0Error::JunkAfterBackslashError(_) => (
                "Unable to parse line continuation",
                "Found token after \\ character",
            ),
        };

        Console::emit(
            crate::output::console::Level::Error,
            file_context,
            prefix,
            message,
            index,
            token.len,
        )
    }
}

impl Emittable for Phase0Error {}
