use std::{
    error::Error,
    fmt::{Display, Formatter},
};

pub type GeneratorResult<T> = Result<T, GeneratorError>;

#[derive(Debug)]
pub enum GeneratorError {
    UnresolvedFuncRefError(String),
    EmptyLabelError(String),
}

impl Error for GeneratorError {}

impl Display for GeneratorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GeneratorError::UnresolvedFuncRefError(name) => {
                write!(f, "Unresolved function reference: {}", name)
            },
            GeneratorError::EmptyLabelError(name) => {
                write!(f, "Label stores no location: {}. Was it ever declared?", name)
            }
        }
    }
}
