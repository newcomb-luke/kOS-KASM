use std::{error::Error, fmt::Display, fmt::Formatter};

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug)]
pub enum ParseError {
    Test,
}

impl Error for ParseError {}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Test => {
                write!(f, "")
            }
        }
    }
}
