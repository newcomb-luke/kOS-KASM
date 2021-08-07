use std::{error::Error, fmt::Display, fmt::Formatter};

use crate::{lexer::token::Token, output::console::Console, FileContext};

pub type PreprocessResult<T> = Result<T, PreprocessError>;
pub type ExpressionResult<T> = Result<T, ExpressionError>;
pub type MacroExpansionResult<T> = Result<T, MacroError>;
pub type DefinitionExpansionResult<T> = Result<T, DefinitionError>;

pub type Phase0Result<T> = Result<T, Phase0Error>;

#[derive(Debug)]
pub enum Phase0Error {
    JunkAfterBackslashError(usize),
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

/*
*
       level: Level,
       file_name: &str,
       source_map: &SourceMap,
       source: &str,
       prefix: &str,
       message: &str,
       index: u32,
       len: u32,
*
*/

#[derive(Debug)]
pub enum PreprocessError {
    DefinitionParseError(String, Box<DefinitionError>),
    MacroParseError(String, usize, Box<MacroError>),
    DefinitionNameCollision(String, usize),
    MacroNameCollision(String, usize),
    InvalidDirectiveTokenType(String, String, usize),
    ExpectedAfterDirective(String, String, usize),
    EndedWithoutClosing(String, String, usize),
    ExtraTokensAfterDirective(String, usize),
    DuplicateLabel(String, usize),
    DirectiveCurrentlyUnsupported(String, usize),
    CannotUndefine(String, usize),
    CannotUnmacro(String, usize),
    InvalidStartOfIf(usize),
    ExpressionParseError(ExpressionError, usize),
    ExpressionEvaluationError(ExpressionError, usize),
    InvalidExpressionResultType(String, String, usize),
    MacroExpansionError(String, usize, MacroError),
    DefinitionExpansionError(String, usize, DefinitionError),
    InvalidIncludeFile(String),
    DirectoryIncludeError(String),
    UnableToReadFile(String, Box<dyn Error>),
    IncludedLexError(String),
    InvalidDirective(String),
    LabelDoesNotExist(String),
}

#[derive(Debug)]
pub enum DefinitionError {
    UnexpectedEOF,
    ExpectedArgument(String),
    ExpectedClosingParen,
    ExpectedArgumentsEnd(String),
    MissingIdentifier,
    DefinitionNotFound(String),
    EmptyDefinition(String),
    EndedWithoutAllArgs,
    InvalidNumberOfArgumentsProvided(usize, usize),
    RecursiveExpansion,
}

#[derive(Debug)]
pub enum MacroError {
    // Parsing
    IncompleteMacroDefinition,
    InvalidNumberOfArguments(String),
    ExpectedArgumentRange(String),
    InvalidArgumentRange((i32, i32)),
    MissingDefaultArgumentValue,
    TokenAfterMacroArguments(String),
    InvalidTokenInDeclaration(String),
    InvalidArgumentReference(String),
    ArgumentReferenceOutOfBounds(i32),
    EndedWithoutClosing,
    MissingIdentifier,
    // Expansion
    MacroNotFound(String),
    InvalidNumberOfArgumentsProvided(usize, usize, usize),
    InnerDefinitionExpansionError(String, usize, DefinitionError),
}

#[derive(Debug)]
pub enum ExpressionError {
    OperatorOnlyValid(String, String),
    OperatorNotValid(String, String),
    IncompleteExpression(String),
    InvalidToken(String),
}

impl Error for PreprocessError {}
impl Error for DefinitionError {}
impl Error for MacroError {}
impl Error for ExpressionError {}

impl Display for PreprocessError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PreprocessError::DefinitionParseError(id, e) => {
                // If the id is empty, then we don't print it
                if id.is_empty() {
                    write!(f, "Error parsing definition: {}", e)
                } else {
                    write!(f, "Error parsing definition {}: {}", id, e)
                }
            }
            PreprocessError::MacroParseError(id, line, e) => {
                // If the id is empty, then we don't print it
                if id.is_empty() {
                    write!(f, "Error parsing macro: {}", e)
                } else {
                    write!(f, "Error parsing macro {}: {}. Line {}", id, e, line)
                }
            }
            PreprocessError::DefinitionNameCollision(s, line) => {
                write!(
                    f,
                    "Cannot create definition {} with same name as macro. Line {}",
                    s, line
                )
            }
            PreprocessError::MacroNameCollision(s, line) => {
                write!(
                    f,
                    "Cannot create macro {} with same name as definition. Line {}",
                    s, line
                )
            }
            PreprocessError::InvalidDirectiveTokenType(token, directive, line) => {
                write!(
                    f,
                    "Invalid token {} for parameter to {} directive. Line {}",
                    token, directive, line
                )
            }
            PreprocessError::ExpectedAfterDirective(expected, directive, line) => {
                write!(
                    f,
                    "Expected {} after {} directive. Line {}",
                    expected, directive, line
                )
            }
            PreprocessError::EndedWithoutClosing(directive, close_directive, line) => {
                write!(
                    f,
                    "{} directive ended unexpectedly without closing {}. Line {}",
                    directive, close_directive, line
                )
            }
            PreprocessError::ExtraTokensAfterDirective(directive, line) => {
                write!(
                    f,
                    "Found extra tokens after {} directive. Line {}",
                    directive, line
                )
            }
            PreprocessError::DuplicateLabel(label, line) => {
                write!(f, "Duplicate label {} defined. Line {}", label, line)
            }
            PreprocessError::DirectiveCurrentlyUnsupported(directive, line) => {
                write!(
                    f,
                    "{} directive is not currently supported in this version of KASM. Line {}",
                    directive, line
                )
            }
            PreprocessError::CannotUndefine(id, line) => {
                write!(
                    f,
                    "Definition {} does not exist, and cannot be undefined. Line {}",
                    id, line
                )
            }
            PreprocessError::CannotUnmacro(id, line) => {
                write!(
                    f,
                    "Macro {} does not exist, and cannot be undefined. Line {}",
                    id, line
                )
            }
            PreprocessError::InvalidStartOfIf(line) => {
                write!(
                    f,
                    "Found if directive on same scope as another if directive. Consider changing to elif or checking nested if's. Line {}",
                    line
                )
            }
            PreprocessError::ExpressionParseError(e, line) => {
                write!(f, "Error while parsing expression: {}. Line {}", e, line)
            }
            PreprocessError::ExpressionEvaluationError(e, line) => {
                write!(f, "Error while evaluating expression: {}. Line {}", e, line)
            }
            PreprocessError::InvalidExpressionResultType(value_for, expected, line) => {
                write!(
                    f,
                    "Expression for {} must evaluate to a {}. Line {}",
                    value_for, expected, line
                )
            }
            PreprocessError::MacroExpansionError(id, line, e) => {
                write!(
                    f,
                    "Error while expanding macro {}: {}. Line {}",
                    id, e, line
                )
            }
            PreprocessError::DefinitionExpansionError(id, line, e) => {
                write!(
                    f,
                    "Error while expanding definition {}: {}. Line {}",
                    id, e, line
                )
            }
            PreprocessError::InvalidIncludeFile(path) => {
                write!(f, "Could not include {}, file does not exist.", path)
            }
            PreprocessError::DirectoryIncludeError(path) => {
                write!(
                    f,
                    "Could not include {}, directories cannot be included",
                    path
                )
            }
            PreprocessError::UnableToReadFile(path, e) => {
                write!(f, "Unable to read file {}: {}", path, e)
            }
            PreprocessError::IncludedLexError(path) => {
                write!(f, "Error lexing included file {}", path)
            }
            PreprocessError::InvalidDirective(s) => {
                write!(f, "Invalid directive found: {}", s)
            }
            PreprocessError::LabelDoesNotExist(s) => {
                write!(f, "Tried to get label {}, but no such label exists", s)
            }
        }
    }
}

impl Display for DefinitionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DefinitionError::UnexpectedEOF => {
                write!(f, "Unexpected EOF while parsing definition")
            }
            DefinitionError::ExpectedArgument(found) => {
                write!(f, "Expected argument, found {}", found)
            }
            DefinitionError::ExpectedClosingParen => {
                write!(f, "Expected closing parenthesis, EOF encountered")
            }
            DefinitionError::ExpectedArgumentsEnd(found) => {
                write!(
                    f,
                    "Expected comma or closing parenthesis, found {} instead",
                    found
                )
            }
            DefinitionError::MissingIdentifier => {
                write!(f, "Declaration missing identifier")
            }
            DefinitionError::DefinitionNotFound(id) => {
                write!(f, "Definition {} referenced before creation", id)
            }
            DefinitionError::EmptyDefinition(id) => {
                write!(f, "Definition {} is empty, and cannot be expanded", id)
            }
            DefinitionError::EndedWithoutAllArgs => {
                write!(
                    f,
                    "Error reading arguments, expected closing parenthesis. Found end of file."
                )
            }
            DefinitionError::InvalidNumberOfArgumentsProvided(provided, expected) => {
                write!(
                    f,
                    "Invalid number of arguments, invocation has {}, expected {}",
                    provided, expected
                )
            }
            DefinitionError::RecursiveExpansion => {
                write!(f, "Cannot have recursive definition expansion")
            }
        }
    }
}

impl Display for MacroError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MacroError::IncompleteMacroDefinition => {
                write!(f, "Incomplete macro definition, expected macro body")
            }
            MacroError::InvalidNumberOfArguments(t) => {
                write!(f, "Expected number of macro arguments, found {}", t)
            }
            MacroError::ExpectedArgumentRange(t) => {
                write!(f, "Expected macro argument range, found {}", t)
            }
            MacroError::InvalidArgumentRange((start, end)) => {
                write!(
                    f,
                    "Invalid macro argument range. Range {}-{} makes no sense",
                    start, end
                )
            }
            MacroError::MissingDefaultArgumentValue => {
                write!(f, "Missing required default argument value")
            }
            MacroError::TokenAfterMacroArguments(t) => {
                if t.is_empty() {
                    write!(f, "Expected newline after macro arguments, found EOF",)
                } else {
                    write!(
                        f,
                        "Expected newline after macro arguments, found {} instead",
                        t
                    )
                }
            }
            MacroError::InvalidTokenInDeclaration(t) => {
                write!(f, "Expected newline or argument range. Found {}", t)
            }
            MacroError::InvalidArgumentReference(t) => {
                if t.is_empty() {
                    write!(
                        f,
                        "Expected int value after & in macro definition, found EOF",
                    )
                } else {
                    write!(
                        f,
                        "Expected int value after & in macro definition, found {}",
                        t
                    )
                }
            }
            MacroError::ArgumentReferenceOutOfBounds(v) => {
                write!(
                    f,
                    "Argument number {} is out of bounds for macro definition",
                    v
                )
            }
            MacroError::EndedWithoutClosing => {
                write!(
                    f,
                    "Macro definition requires closing .endmacro directive. Reached EOF"
                )
            }
            MacroError::MissingIdentifier => {
                write!(f, "Macro missing identifier")
            }
            MacroError::MacroNotFound(id) => {
                write!(f, "Macro {} referenced before creation", id)
            }
            MacroError::InvalidNumberOfArgumentsProvided(provided, required, line) => {
                write!(
                    f,
                    "Invalid number of arguments for macro expansion. Invocation has {}, at least {} required. Line {}",
                    provided, required, line
                )
            }
            MacroError::InnerDefinitionExpansionError(id, line, e) => {
                write!(
                    f,
                    "Could not expand definition {}, {}. Line {}",
                    id, e, line
                )
            }
        }
    }
}

impl Display for ExpressionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpressionError::OperatorOnlyValid(operator, correct) => {
                write!(
                    f,
                    "{} operator only valid on the type {}",
                    operator, correct
                )
            }
            ExpressionError::OperatorNotValid(operator, wrong) => {
                write!(f, "{} operator not valid on the type {}", operator, wrong)
            }
            ExpressionError::IncompleteExpression(s) => {
                write!(f, "{}", s)
            }
            ExpressionError::InvalidToken(t) => {
                write!(f, "Invalid token in expression: {:?}", t)
            }
        }
    }
}
