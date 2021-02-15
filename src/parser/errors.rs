use std::{error::Error, fmt::Display, fmt::Formatter};

use crate::ExpressionError;

pub type ParseResult<T> = Result<T, ParseError>;
pub type InstructionParseResult<T> = Result<T, InstructionParseError>;

#[derive(Debug)]
pub enum ParseError {
    InstructionParseFailed(InstructionParseError, usize),
    DuplicateLabelError(String, usize),
    TokenOutsideFunctionError(usize),
    UndefinedLabelError(String),
}

#[derive(Debug)]
pub enum InstructionParseError {
    InvalidInstructionError(String),
    ExpectedOperandError,
    NumOperandsMismatchError(usize, usize),
    InvalidOperandTypeError(usize, String),
    ExtraTokensInOperandError(String),
    ExpressionParseFailedError(usize, ExpressionError),
    ExpressionEvalFailedError(usize, ExpressionError),
    InternalOperandNotAcceptedError,
    InternalOperandTooLargeError,
    IntOperandTooLargeError(String),
}

impl Error for ParseError {}
impl Error for InstructionParseError {}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InstructionParseFailed(e, line) => {
                write!(f, "Error parsing instruction: {}. Line {}", e, line)
            }
            ParseError::DuplicateLabelError(label_id, line) => {
                write!(
                    f,
                    "Duplicate label name used: {}, second use on line {}",
                    label_id, line
                )
            }
            ParseError::TokenOutsideFunctionError(line) => {
                write!(f, "Token found outside function, line {}", line)
            }
            ParseError::UndefinedLabelError(id) => {
                write!(f, "Undefined label used {}", id)
            }
        }
    }
}

impl Display for InstructionParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InstructionParseError::InvalidInstructionError(id) => {
                write!(f, "Identifier {} is not a valid instruction", id)
            }
            InstructionParseError::ExpectedOperandError => {
                write!(f, "Expected an operand after comma, but found none")
            }
            InstructionParseError::NumOperandsMismatchError(expected, found) => {
                write!(
                    f,
                    "Instruction requires {} operands, but {} were supplied",
                    expected, found
                )
            }
            InstructionParseError::InvalidOperandTypeError(op_number, accepted_list) => {
                write!(
                    f,
                    "Operand {} is of the wrong type. Accepted types are:{}",
                    op_number, accepted_list
                )
            }
            InstructionParseError::ExtraTokensInOperandError(operand_kind_str) => {
                write!(
                    f,
                    "Found extra tokens in operand. If operand is a {0}, it must only contain the {0}",
                    operand_kind_str
                )
            }
            InstructionParseError::ExpressionParseFailedError(op_number, e) => {
                write!(
                    f,
                    "Expected expression as operand {}, expression parsing failed: {}",
                    op_number, e
                )
            }
            InstructionParseError::ExpressionEvalFailedError(op_number, e) => {
                write!(
                    f,
                    "Expected expression as operand {}, expression evaluation failed: {}",
                    op_number, e
                )
            }
            InstructionParseError::InternalOperandNotAcceptedError => {
                write!(f, "This should never be printed")
            }
            InstructionParseError::InternalOperandTooLargeError => {
                write!(f, "This should never be printed")
            }
            InstructionParseError::IntOperandTooLargeError(accepted) => {
                write!(
                    f,
                    "Integer operand as accepted, but is too large to be stored. Acceptable types are: {}",
                    accepted
                )
            }
        }
    }
}
