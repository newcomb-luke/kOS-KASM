use std::fmt;
use std::io::Write;

use termcolor::{Color, ColorSpec, StandardStream, WriteColor};

use crate::lexer::token::Token;

pub type KASMResult<T> = Result<T, ()>;

static WARNING_COLOR: Color = Color::Yellow;
static ERROR_COLOR: Color = Color::Red;
static NOTE_COLOR: Color = Color::Green;
static HELP_COLOR: Color = Color::Cyan;
static PLAIN_WHITE: Color = Color::Rgb(255, 255, 255);
static PROMPT_COLOR: Color = Color::Blue;

#[derive(Debug, Clone)]
pub enum KASMError {
    Assembly(AssemblyError),
    Internal(InternalError),
}

impl From<AssemblyError> for KASMError {
    fn from(err: AssemblyError) -> Self {
        KASMError::Assembly(err)
    }
}

impl From<InternalError> for KASMError {
    fn from(err: InternalError) -> Self {
        KASMError::Internal(err)
    }
}

#[derive(Debug)]
pub struct ErrorManager {
    errors: Vec<KASMError>,
}

impl ErrorManager {
    pub fn new() -> Self {
        ErrorManager { errors: Vec::new() }
    }

    pub fn add(&mut self, err: KASMError) {
        self.errors.push(err);
    }

    pub fn add_assembly(&mut self, err: AssemblyError) {
        self.errors.push(KASMError::Assembly(err));
    }

    pub fn add_internal(&mut self, err: InternalError) {
        self.errors.push(KASMError::Internal(err));
    }

    /// Emits any errors and warnings that have been generated and store in th
    pub fn emit(&mut self, files: &Vec<SourceFile>) -> std::io::Result<bool> {
        if self.errors.len() > 0 {
            let mut had_fatal = false;

            for error in self.errors.drain(..) {
                if error.is_fatal() {
                    had_fatal = true;
                }

                match error {
                    KASMError::Assembly(err) => err.emit(files)?,
                    KASMError::Internal(err) => err.emit()?,
                }
            }

            Ok(had_fatal)
        } else {
            Ok(false)
        }
    }

    pub fn errors(&self) -> &Vec<KASMError> {
        &self.errors
    }
}

impl KASMError {
    /// Returns true if this error is considered fatal or not
    pub fn is_fatal(&self) -> bool {
        match self {
            KASMError::Assembly(err) => err.is_fatal(),
            KASMError::Internal(_) => true,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Level {
    Bug,
    Error,
    Warning,
    Note,
    Help,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_str().fmt(f)
    }
}

impl Level {
    fn color(&self) -> ColorSpec {
        let mut spec = ColorSpec::new();

        match self {
            Level::Bug | Level::Error => {
                spec.set_fg(Some(ERROR_COLOR)).set_intense(true);
            }
            Level::Warning => {
                spec.set_fg(Some(WARNING_COLOR)).set_intense(true);
            }
            Level::Note => {
                spec.set_fg(Some(NOTE_COLOR)).set_intense(true);
            }
            Level::Help => {
                spec.set_fg(Some(HELP_COLOR)).set_intense(true);
            }
        }
        spec
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Level::Bug => "error: internal assembler error",
            Level::Error => "error",
            Level::Warning => "warning",
            Level::Note => "note",
            Level::Help => "help",
        }
    }

    /// Returns true if this error level is considered fatal
    pub fn is_fatal(&self) -> bool {
        match self {
            Level::Bug => false,
            Level::Error => true,
            Level::Warning => false,
            Level::Note => false,
            Level::Help => false,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ErrorData {
    pub prefix: &'static str,
    pub message: &'static str,
    pub level: Level,
}

impl ErrorData {
    pub fn new(prefix: &'static str, message: &'static str, level: Level) -> Self {
        ErrorData {
            prefix,
            message,
            level,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ErrorKind {
    ShouldNotBeShown,

    JunkAfterBackslash,
    JunkDirective,
    TokenParse,
    JunkFloat,

    UnexpectedEndOfExpression,
    MissingClosingExpressionParen,
    InvalidTokenExpression,

    IntegerParse,
    FloatParse,

    MissingDirectiveIdentifier,
    ExpectedDirectiveIdentifier,
    InvalidTokenDirectiveArguments,
    NotCommaOrParenDirectiveArguments,
    UnexpectedEndOfDirectiveArguments,
    WarnEmptyDirectiveArguments,
    WarnEmptyDirectiveExpansionWithArgs,

    MissingUndefIdentifier,
    ExpectedUndefIdentifier,
}

impl ErrorKind {
    pub fn error_data(&self) -> ErrorData {
        match self {
            ErrorKind::ShouldNotBeShown => ErrorData::new(
                "This error should never be shown, and is a placeholder",
                "This error was shown here!!!",
                Level::Error,
            ),
            ErrorKind::JunkAfterBackslash => ErrorData::new(
                "Unable to parse line continuation",
                "Found token after \\ character",
                Level::Error,
            ),
            ErrorKind::JunkDirective => ErrorData::new(
                "Error parsing directive",
                "Not a valid directive name",
                Level::Error,
            ),
            ErrorKind::TokenParse => {
                ErrorData::new("Error parsing token", "Invalid token found", Level::Error)
            }
            ErrorKind::JunkFloat => ErrorData::new(
                "Error parsing float",
                "Found invalid character(s)",
                Level::Error,
            ),
            ErrorKind::UnexpectedEndOfExpression => ErrorData::new(
                "Unable to parse expression",
                "Expected more tokens to complete expression",
                Level::Error,
            ),
            ErrorKind::MissingClosingExpressionParen => ErrorData::new(
                "Unable to parse expression",
                "Expected closing )",
                Level::Error,
            ),
            ErrorKind::InvalidTokenExpression => ErrorData::new(
                "Unable to parse expression",
                "Found invalid token(s)",
                Level::Error,
            ),
            ErrorKind::IntegerParse => ErrorData::new(
                "Unable to parse integer literal",
                "Found invalid character(s)",
                Level::Error,
            ),
            ErrorKind::FloatParse => ErrorData::new(
                "Unable to parse float literal",
                "Found invalid character(s)",
                Level::Error,
            ),
            ErrorKind::MissingDirectiveIdentifier => ErrorData::new(
                "Error while parsing directive declaration",
                "Expected an identifier, found end of file",
                Level::Error,
            ),
            ErrorKind::ExpectedDirectiveIdentifier => ErrorData::new(
                "Error while parsing directive declaration",
                "Expected an identifier, found invalid token",
                Level::Error,
            ),
            ErrorKind::InvalidTokenDirectiveArguments => ErrorData::new(
                "Error while parsing directive arguments",
                "Expected an argument or closing )",
                Level::Error,
            ),
            ErrorKind::NotCommaOrParenDirectiveArguments => ErrorData::new(
                "Error while parsing directive arguments",
                "Expected a , or closing )",
                Level::Error,
            ),
            ErrorKind::UnexpectedEndOfDirectiveArguments => ErrorData::new(
                "Error while parsing directive arguments",
                "End of file reached after token",
                Level::Error,
            ),
            ErrorKind::WarnEmptyDirectiveArguments => ErrorData::new(
                "Empty arguments list",
                "Did you mean to add arguments?",
                Level::Warning,
            ),
            ErrorKind::WarnEmptyDirectiveExpansionWithArgs => ErrorData::new(
                "Empty definition expansion",
                "Definition with arguments, but no use of them",
                Level::Warning,
            ),
            ErrorKind::MissingUndefIdentifier => ErrorData::new(
                "Error while parsing undefine",
                "Expected an identifier, found end of file",
                Level::Error,
            ),
            ErrorKind::ExpectedUndefIdentifier => ErrorData::new(
                "Error while parsing undefine",
                "Expected an identifier, found invalid token",
                Level::Error,
            ),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum InternalError {
    ErrorDisplayError,
    FindErrorTokenError,
}

#[derive(Debug, Copy, Clone)]
pub struct AssemblyError {
    kind: ErrorKind,
    token: Token,
    error_data: ErrorData,
}

impl InternalError {
    pub fn emit(&self) -> std::io::Result<()> {
        let mut stream = StandardStream::stdout(termcolor::ColorChoice::Auto);

        let mut message_color = Level::Bug.color();
        message_color.set_bold(true);

        let mut white_color = ColorSpec::new();
        white_color.set_fg(Some(PLAIN_WHITE));
        white_color.set_bold(true);

        let message = match self {
            InternalError::ErrorDisplayError => "Unable to display assembly error!",
            InternalError::FindErrorTokenError => "Unable to find location of error in token map",
        };

        stream.set_color(&message_color)?;

        write!(stream, "{}", Level::Bug)?;

        stream.set_color(&white_color)?;

        writeln!(stream, ": {}", message)?;

        Ok(())
    }
}

impl AssemblyError {
    pub fn new(kind: ErrorKind, token: Token) -> Self {
        Self {
            kind,
            token,
            error_data: kind.error_data(),
        }
    }

    pub fn emit(&self, files: &Vec<SourceFile>) -> std::io::Result<()> {
        if let Some(file) = files.get(self.token.file_id as usize) {
            self.emit_normal(file)?;
        } else {
            InternalError::FindErrorTokenError.emit()?;
        }

        Ok(())
    }

    fn emit_normal(&self, file: &SourceFile) -> std::io::Result<()> {
        let error_data = self.error_data;
        let level = error_data.level;
        let prefix = error_data.prefix;
        let message = error_data.message;
        let len = self.token.len;
        let index = self.token.source_index;

        if let Some(line) = file.get_line(index) {
            let original_line = &file.source()[line.start as usize..line.end as usize];

            let line_string = original_line.replace("\t", "    ");

            let column = (index - line.start) + 3 * original_line.matches("\t").count() as u32;
            let line_num_string = format!("{}", line.num);

            let mut stream = StandardStream::stdout(termcolor::ColorChoice::Auto);

            let regular_color = ColorSpec::new();

            let mut message_color = level.color();
            message_color.set_bold(true);

            let mut white_color = ColorSpec::new();
            white_color.set_fg(Some(PLAIN_WHITE));
            white_color.set_bold(true);

            let mut prompt_color = ColorSpec::new();
            prompt_color.set_fg(Some(PROMPT_COLOR));
            prompt_color.set_intense(true);
            prompt_color.set_bold(true);

            stream.set_color(&message_color)?;

            write!(stream, "{}", level)?;

            stream.set_color(&white_color)?;

            writeln!(stream, ": {}: {}", prefix, message)?;

            stream.set_color(&prompt_color)?;

            write!(stream, "  --> ")?;

            stream.set_color(&regular_color)?;

            writeln!(stream, "{}:{}:{}", file.name(), line.num, column)?;

            stream.set_color(&prompt_color)?;

            writeln!(stream, "{:<width$} | ", "", width = line_num_string.len())?;

            write!(stream, "{} | ", line_num_string)?;

            stream.set_color(&regular_color)?;

            writeln!(stream, "{}", line_string)?;

            stream.set_color(&prompt_color)?;

            write!(stream, "{:<width$} | ", "", width = line_num_string.len())?;

            write!(stream, "{:<width$}", "", width = column as usize)?;

            stream.set_color(&message_color)?;

            for _ in 0..len {
                write!(stream, "^")?;
            }

            writeln!(stream, " {}", message)?;

            stream.set_color(&prompt_color)?;

            writeln!(stream, "{:<width$} | ", "", width = line_num_string.len())?;

            writeln!(stream, "")?;
        } else {
            InternalError::ErrorDisplayError.emit()?;
        }

        Ok(())
    }

    /// Returns the error kind of this error
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// Returns if this error is fatal or not
    pub fn is_fatal(&self) -> bool {
        self.error_data.level.is_fatal()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Line {
    pub start: u32,
    pub end: u32,
    pub num: u32,
}

impl Line {
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end, num: 0 }
    }

    pub fn with_num(start: u32, end: u32, num: u32) -> Self {
        Self { start, end, num }
    }
}

#[derive(Debug, Clone)]
pub struct SourceFile {
    name: String,
    source: String,
    lines: Vec<Line>,
}

impl SourceFile {
    pub fn new(name: String, source: String) -> Self {
        // Generate the line maps
        let lines = Self::generate(&source);

        Self {
            name,
            source,
            lines,
        }
    }

    /// Returns the start and end positions of the line the index is in
    pub fn get_line(&self, index: u32) -> Option<Line> {
        for line in self.lines.iter() {
            if line.start <= index && index <= line.end {
                return Some(*line);
            }
        }

        None
    }

    /// Generates a source map for the given source
    fn generate(source: &str) -> Vec<Line> {
        let mut lines = Vec::new();
        let mut current_index = 0;

        // Split the source by lines
        for (index, source_line) in source.split('\n').enumerate() {
            let start = current_index;
            // Increment the current index to account for this line
            current_index += source_line.len() + 1;

            let line = Line::with_num(start as u32, (current_index - 1) as u32, (index + 1) as u32);

            lines.push(line);
        }

        lines
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn source(&self) -> &String {
        &self.source
    }
}
