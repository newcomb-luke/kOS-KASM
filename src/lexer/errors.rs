use super::Token;
use std::{error::Error, fmt::Display, fmt::Formatter};

pub type LexResult<T> = Result<T, LexError>;
pub type LiteralResult<T> = Result<T, LiteralParseError>;

#[derive(Debug)]
pub enum LexError {
    TokenAfterLineContinue(Token),
    ExpectedChar(String, String),
    UnexpectedChar(char),
    LoneChar(char),
    TrailingEscape,
    InvalidEscapedChar(char),
    LiteralError(LiteralParseError),
    ErrorWrapper(String, usize, Box<dyn Error>),
}

#[derive(Debug)]
pub enum LiteralParseError {
    ExpectedBinary,
    ExpectedHex,
    InvalidDouble(String),
    InvalidBinary(String),
    InvalidHex(String),
    InvalidInt(String),
    BinaryTooLarge(String),
    HexTooLarge(String),
    IntTooLarge(String),
}

impl Error for LexError {}
impl Error for LiteralParseError {}

impl Display for LexError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LexError::TokenAfterLineContinue(token) => {
                write!(
                    f,
                    "Error parsing, \\ should only be followed by a newline. Found: {}",
                    token.as_str()
                )
            }
            LexError::ExpectedChar(found, expected) => {
                write!(f, "Found {}, expected {}", found, expected)
            }
            LexError::UnexpectedChar(c) => {
                write!(f, "Unexpected char {} while parsing token", c)
            }
            LexError::LoneChar(c) => {
                write!(f, "Found line {} char", c)
            }
            LexError::TrailingEscape => {
                write!(
                    f,
                    "Found trailing \\, consider \\\\ or finishing the escape sequence"
                )
            }
            LexError::InvalidEscapedChar(c) => {
                write!(f, "\\{} is not a valid escape sequence", c)
            }
            LexError::LiteralError(e) => {
                write!(f, "{}", e)
            }
            LexError::ErrorWrapper(file, line, e) => {
                write!(
                    f,
                    "Error lexing input in file {} line {}. {}.",
                    file, line, e
                )
            }
        }
    }
}

impl Display for LiteralParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LiteralParseError::ExpectedBinary => {
                write!(f, "Found characters 0b, expected binary literal")
            }
            LiteralParseError::ExpectedHex => {
                write!(f, "Found characters 0x, expected hex literal")
            }
            LiteralParseError::InvalidDouble(d) => {
                write!(f, "Invalid double literal {}", d)
            }
            LiteralParseError::InvalidBinary(b) => {
                write!(f, "Invalid binary literal {}", b)
            }
            LiteralParseError::InvalidHex(x) => {
                write!(f, "Invalid hex literal {}", x)
            }
            LiteralParseError::InvalidInt(i) => {
                write!(f, "Invalid int literal {}", i)
            }
            LiteralParseError::BinaryTooLarge(b) => {
                write!(
                    f,
                    "Binary literal {} too large to fit into 32-bit integer",
                    b
                )
            }
            LiteralParseError::HexTooLarge(x) => {
                write!(
                    f,
                    "Hexadecimal literal {} too large to fit into 32-bit integer",
                    x
                )
            }
            LiteralParseError::IntTooLarge(i) => {
                write!(
                    f,
                    "Integer literal {} too large to fit into 32-bit integer",
                    i
                )
            }
        }
    }
}
