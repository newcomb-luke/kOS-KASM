use std::fmt;
use std::io::Write;

use termcolor::{Color, ColorSpec, StandardStream, WriteColor};

use crate::lexer::token::Token;

static WARNING_COLOR: Color = Color::Yellow;
static ERROR_COLOR: Color = Color::Red;
static NOTE_COLOR: Color = Color::Green;
static HELP_COLOR: Color = Color::Cyan;
static PLAIN_WHITE: Color = Color::Rgb(255, 255, 255);
static PROMPT_COLOR: Color = Color::Blue;

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
}

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

pub enum ErrorKind {
    JunkAfterBackslash,
    TokenParse,
    JunkFloat,
}

impl ErrorKind {
    pub fn error_data(&self) -> ErrorData {
        match self {
            ErrorKind::JunkAfterBackslash => ErrorData::new(
                "Unable to parse line continuation",
                "Found token after \\ character",
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
        }
    }
}

pub enum InternalError {
    ErrorDisplayError,
    FindErrorTokenError,
}

pub struct Error {
    kind: ErrorKind,
    token_index: u32,
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

impl Error {
    pub fn new(kind: ErrorKind, token_index: u32) -> Self {
        Error { kind, token_index }
    }

    pub fn emit(&self, token_map: &TokenMap, tokens: &Vec<Token>) -> std::io::Result<()> {
        let mut index = 0;
        let mut error_token = None;
        let error_data = self.kind.error_data();

        for (iter_index, token) in tokens.iter().enumerate() {
            if iter_index as u32 == self.token_index {
                error_token = Some(token);
                break;
            }

            index += token.len;
        }

        let file_context = token_map.file_at(index);

        if let (Some((file_context, _)), Some(token)) = (file_context, error_token) {
            Error::emit_normal(file_context, &error_data, index, token)?;
        } else {
            InternalError::FindErrorTokenError.emit()?;
        }

        Ok(())
    }

    fn emit_normal(
        file_context: &FileContext,
        error_data: &ErrorData,
        index: u32,
        token: &Token,
    ) -> std::io::Result<()> {
        let level = error_data.level;
        let prefix = error_data.prefix;
        let message = error_data.message;
        let len = token.len;

        if let Some((line_num, line_start, line_end)) = file_context.source_map().get_line(index) {
            let original_line = &file_context.source()[line_start as usize..line_end as usize];

            let line_string = original_line.replace("\t", "    ");

            let column = (index - line_start) + 3 * original_line.matches("\t").count() as u32;
            let line_num_string = format!("{}", line_num);

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

            writeln!(stream, "{}:{}:{}", file_context.name(), line_num, column)?;

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
}

/// A struct used to store where lines in the source are, for warning and error messages
#[derive(Debug, Clone)]
pub struct SourceMap {
    lines: Vec<(u32, u32)>,
}

impl SourceMap {
    /// Generates a source map for the given source
    pub fn generate(source: &str) -> Self {
        let mut lines = Vec::new();
        let mut last_start = 0;
        let mut current_index = 0;

        for char in source.chars() {
            if char == '\n' {
                let line = (last_start, current_index);
                last_start = current_index + 1;

                lines.push(line);
            }

            current_index += 1;
        }

        lines.push((last_start, current_index));

        SourceMap { lines }
    }

    /// Returns the start and end positions of the line the index is in
    pub fn get_line(&self, index: u32) -> Option<(u32, u32, u32)> {
        for (line_num, line) in self.lines.iter().enumerate() {
            if line.0 <= index && index <= line.1 {
                let data = ((line_num + 1) as u32, line.0, line.1);

                return Some(data);
            }
        }

        None
    }
}

#[derive(Debug, Clone)]
pub struct FileContext {
    name: String,
    source: String,
    source_map: SourceMap,
}

impl FileContext {
    pub fn new(name: String, source: String) -> Self {
        let source_map = SourceMap::generate(&source);

        Self {
            name,
            source,
            source_map,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn source(&self) -> &String {
        &self.source
    }

    pub fn source_map(&self) -> &SourceMap {
        &self.source_map
    }

    pub fn get_line(&self, index: u32) -> Option<(u32, u32, u32)> {
        self.source_map.get_line(index)
    }
}

/// A struct to map absolute token indexes to files
pub struct TokenMap {
    file_contexts: Vec<FileContext>,
    file_ranges: Vec<(u32, u32)>,
    current_end: u32,
}

impl TokenMap {
    pub fn new() -> Self {
        TokenMap {
            file_contexts: Vec::new(),
            file_ranges: Vec::new(),
            current_end: 0,
        }
    }

    pub fn add(&mut self, file_context: FileContext) {
        let size = file_context.source().len();
        let new_end = self.current_end + size as u32;

        let range = (self.current_end, new_end);

        self.file_contexts.push(file_context);
        self.file_ranges.push(range);

        self.current_end = new_end;
    }

    pub fn file_at(&self, index: u32) -> Option<(&FileContext, u32)> {
        let mut file_context = None;
        let mut offset = 0;

        for (file_index, range) in self.file_ranges.iter().enumerate() {
            if range.0 <= index && index <= range.1 {
                file_context = self.file_contexts.get(file_index);
                offset = range.0;

                break;
            }
        }

        if let Some(file) = file_context {
            Some((file, index - offset))
        } else {
            None
        }
    }
}
