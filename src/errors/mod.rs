use std::fmt::Display;
use std::io::Write;
use std::rc::Rc;
use std::sync::RwLock;
use std::{path::PathBuf, sync::Mutex};

use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
// To-do list:
// * Trim code to the right of the area of interest, we don't want comments clogging it up
//

static WARNING_COLOR: Color = Color::Yellow;
static ERROR_COLOR: Color = Color::Red;
static NOTE_COLOR: Color = Color::Green;
static HELP_COLOR: Color = Color::Cyan;
static PLAIN_WHITE: Color = Color::Rgb(255, 255, 255);
static PROMPT_COLOR: Color = Color::Blue;

pub struct DiagnosticBuilder<'a> {
    diagnostic: Diagnostic,
    handler: &'a Handler,
}

impl<'a> DiagnosticBuilder<'a> {
    /// For internal use only, creates a new DiagnosticBuilder. For clients, the struct_* methods
    /// on a Session or Handler should be used instead.
    pub(crate) fn new(handler: &'a Handler, level: Level, message: String) -> Self {
        let diagnostic = Diagnostic {
            level,
            message,
            primary: None,
            spans: Vec::new(),
            children: Vec::new(),
        };

        Self {
            diagnostic,
            handler,
        }
    }

    pub fn set_primary_span(&mut self, span: Span) -> &mut Self {
        self.diagnostic.primary = Some(span);

        self
    }

    pub fn span_label(&mut self, span: Span, label: String) -> &mut Self {
        self.diagnostic.spans.push((span, label));

        self
    }

    /// Adds a note message to the diagnostic
    pub fn note(&mut self, message: String) -> &mut Self {
        let subd = SubDiagnostic::new(Level::Note, message);
        self.diagnostic.children.push(subd);

        self
    }

    /// Adds a help message to the diagnostic
    pub fn help(&mut self, message: String) -> &mut Self {
        let subd = SubDiagnostic::new(Level::Help, message);
        self.diagnostic.children.push(subd);

        self
    }

    /// Queues this diagnostic to be emitted by the inner Handler/Emitter
    pub fn emit(&mut self) {
        if self.diagnostic.level == Level::Warning {
            self.handler.warn(self.diagnostic.clone());
        } else {
            self.handler.error(self.diagnostic.clone());
        }

        // Mark this as cancelled so that it can be safely dropped
        self.cancel();
    }

    /// Sets this DiagnosticBuilder as cancelled, meaning that it is safe to be dropped
    pub fn cancel(&mut self) {
        self.diagnostic.level = Level::Cancelled;
    }

    /// Returns true if this was cancelled, false otherwise
    pub fn cancelled(&self) -> bool {
        self.diagnostic.level == Level::Cancelled
    }
}

impl<'a> Drop for DiagnosticBuilder<'a> {
    fn drop(&mut self) {
        // DiagnosticBuilders are sort of bombs if dropped. This had better either be emitted, or
        // cancelled. If not, we emit a bug error.
        if !self.cancelled() {
            let mut db = DiagnosticBuilder::new(
                self.handler,
                Level::Bug,
                "the following error was constructed but not emitted".to_string(),
            );

            db.emit();
            self.emit();
        }
    }
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub level: Level,
    pub message: String,
    pub primary: Option<Span>,
    pub spans: Vec<(Span, String)>,
    pub children: Vec<SubDiagnostic>,
}

#[derive(Debug, Clone)]
pub struct SubDiagnostic {
    pub level: Level,
    pub message: String,
}

impl SubDiagnostic {
    /// Creates a new sub diagnostic
    pub fn new(level: Level, message: String) -> Self {
        Self { level, message }
    }
}

pub struct Emitter {
    flags: HandlerFlags,
    source_manger: Rc<RwLock<SourceManager>>,
}

impl Emitter {
    pub fn new(flags: HandlerFlags, source_manger: Rc<RwLock<SourceManager>>) -> Self {
        Self {
            flags,
            source_manger,
        }
    }

    fn color_choice(&self) -> ColorChoice {
        if self.flags.colored_output {
            ColorChoice::Auto
        } else {
            ColorChoice::Never
        }
    }

    fn get_stderr(&self) -> StandardStream {
        StandardStream::stderr(self.color_choice())
    }

    pub fn emit_diagnostic(&self, diagnostic: &Diagnostic) {
        let mut stream = self.get_stderr();

        let level_msg = diagnostic.level.as_styled_string();

        match self.emit_styled_string(&mut stream, &level_msg) {
            Err(e) => {
                panic!("Failed to emit error: {}", e);
            }
            _ => {}
        }

        let styled_string =
            StyledString::new(format!(": {}", diagnostic.message), Style::MainHeaderMsg);

        match self.emit_styled_string(&mut stream, &styled_string) {
            Err(e) => {
                panic!("Failed to emit error: {}", e);
            }
            _ => {}
        }
        eprintln!("");

        if let Some(primary) = &diagnostic.primary {
            let source_location = self.get_source_location(primary);
            let snippet = self.span_to_snippet(primary);

            let extra_spacer = diagnostic.spans.len() == 0;

            match self.emit_snippet(
                &mut stream,
                primary,
                source_location,
                &snippet,
                diagnostic.level,
                None,
                extra_spacer,
                true,
            ) {
                Err(e) => {
                    panic!("Failed to emit snippet: {}", e);
                }
                _ => {}
            }
        }

        let styled_dots = StyledString::new("...".to_string(), Style::LineAndColumn);

        if diagnostic.primary.is_some() && diagnostic.spans.len() > 0 {
            // We need the special dots
            self.emit_styled_string(&mut stream, &styled_dots)
                .expect("Failed to emit ...");
            eprint!("\n");
        }

        for (index, (span, label)) in diagnostic.spans.iter().enumerate() {
            let mut extra_spacer = true;

            if index + 1 < diagnostic.spans.len() {
                // Put the dots
                self.emit_styled_string(&mut stream, &styled_dots)
                    .expect("Failed to emit ...");
                eprint!("\n");

                extra_spacer = false;
            }

            let snippet = self.span_to_snippet(&span);
            let source_location = self.get_source_location(&span);

            self.emit_snippet(
                &mut stream,
                span,
                source_location,
                &snippet,
                diagnostic.level,
                Some(label),
                extra_spacer,
                diagnostic.primary.is_none(),
            )
            .expect("Failed to emit snippet");
        }

        for sub_diagnostic in diagnostic.children.iter() {
            let styled_leader = StyledString::new(String::from(" = "), Style::LineAndColumn);
            let styled_level = sub_diagnostic.level.as_styled_string();
            let styled_message =
                StyledString::new(format!(": {}", sub_diagnostic.message), Style::NoStyle);

            self.emit_styled_string(&mut stream, &styled_leader)
                .expect("Failed to emit ...");

            self.emit_styled_string(&mut stream, &styled_level)
                .expect("Failed to emit ...");

            self.emit_styled_string(&mut stream, &styled_message)
                .expect("Failed to emit ...");
        }
    }

    fn emit_snippet(
        &self,
        stream: &mut StandardStream,
        span: &Span,
        source_location: (String, usize, usize),
        snippet: &Snippet,
        level: Level,
        label: Option<&str>,
        extra_spacer: bool,
        display_file: bool,
    ) -> std::io::Result<()> {
        let (path, line_num, col) = source_location;

        let line_num_str = format!("{}", line_num);
        let line_num_width = line_num_str.len();

        //   --> src/main.kasm:2:4
        if display_file {
            let styled_arrow = StyledString::new(
                format!("{:spaces$}--> ", "", spaces = line_num_width),
                Style::LineNumber,
            );

            // Emit the:
            //    -->
            self.emit_styled_string(stream, &styled_arrow)?;

            // src/main.kasm:2:4
            eprintln!(" {}:{}:{}", path, line_num, col);
        }

        let vert_bar = StyledString::new(
            format!("{:spaces$} |", "", spaces = line_num_width),
            Style::LineAndColumn,
        );

        //     |
        self.emit_styled_string(stream, &vert_bar)?;
        eprint!("\n");

        // 200 |
        self.emit_styled_string(stream, &self.struct_line_num(line_num))?;

        //   push NOT_ALLOWED
        eprintln!("{}", &snippet.line);

        //     |
        self.emit_styled_string(stream, &vert_bar)?;

        //    ^^^^^^^^^^^^
        // Print the spaces
        eprint!("{:spaces$} ", "", spaces = col);

        // Print the ^'s
        // If anyone reading this knows a better way, let me know. ^ is a special character in
        // formatting strings, so.
        stream.set_color(&Style::Level(level).to_spec())?;

        for _ in 0..(span.end - span.start) {
            write!(stream, "^")?;
        }

        write!(stream, " ")?;

        // Add the label if it exists
        if let Some(label) = label {
            write!(stream, "{}", label)?;
        }

        eprint!("\n");

        if extra_spacer {
            //     |
            self.emit_styled_string(stream, &vert_bar)?;
            eprint!("\n");
        }

        Ok(())
    }

    // Constructs a StyledString that contains this line number but formatted like a diagnostic:
    //
    // Ex:
    //
    //  243 |
    fn struct_line_num(&self, line_num: usize) -> StyledString {
        StyledString::new(format!("{} | ", line_num), Style::LineNumber)
    }

    fn get_source_location(&self, span: &Span) -> (String, usize, usize) {
        let file_id = span.file;

        match self
            .source_manger
            .read()
            .unwrap()
            .get_by_id(file_id as usize)
        {
            Some(source_file) => source_file.get_source_location(span),
            None => {
                panic!("Failed to get source location of span");
            }
        }
    }

    fn span_to_snippet(&self, span: &Span) -> Snippet {
        let file_id = span.file;

        match self
            .source_manger
            .read()
            .unwrap()
            .get_by_id(file_id as usize)
        {
            Some(source_file) => source_file.span_to_snippet(span),
            None => {
                panic!("Failed to convert span to snippet");
            }
        }
    }

    pub fn emit_styled_string(
        &self,
        stream: &mut StandardStream,
        styled_string: &StyledString,
    ) -> std::io::Result<()> {
        let color_spec = styled_string.style.to_spec();

        stream.set_color(&color_spec)?;

        write!(stream, "{}", styled_string.text)?;

        stream.set_color(&ColorSpec::new())?;

        Ok(())
    }
}

pub struct StyledString {
    text: String,
    style: Style,
}

impl StyledString {
    pub fn new(text: String, style: Style) -> Self {
        Self { text, style }
    }
}

pub struct SourceLocation {
    pub path: PathBuf,
}

#[derive(Debug, Clone, Copy)]
pub enum Style {
    MainHeaderMsg,
    Level(Level),
    NoStyle,
    LineNumber,
    LineAndColumn,
}

impl Style {
    /// Converts a Style into a ColorSpec for colored output
    pub fn to_spec(&self) -> ColorSpec {
        match self {
            Style::NoStyle => ColorSpec::new(),
            Style::MainHeaderMsg => {
                let mut main_msg = ColorSpec::new();
                main_msg.set_fg(Some(PLAIN_WHITE));
                main_msg.set_bold(true);

                main_msg
            }
            Style::LineNumber | Style::LineAndColumn => {
                let mut line_num = ColorSpec::new();
                line_num.set_fg(Some(PROMPT_COLOR));
                line_num.set_intense(true);
                line_num.set_bold(true);

                line_num
            }
            Style::Level(level) => level.color(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct HandlerFlags {
    /// If the output should be colored or not. This should be false when the output is redirected
    /// into a file, for example.
    pub colored_output: bool,
    /// Warnings can be disabled by command-line flags
    pub emit_warnings: bool,
    /// This flag means if this Handler should actually print anything at all. This should probably
    /// be set when this is being used as a library
    pub quiet: bool,
}

// This is needed so that certain parts of the Handler can be put behind a Mutex, so that they can
// be mutably changed without Handler needing to be mutably borrowed, and so that it could
// theoretically be safe across threads should that day come.
pub(crate) struct HandlerInner {
    /// The inner emitter that actually emits the Diagnostics
    pub emitter: Emitter,
    // pub source_manager: Rc<RwLock<SourceManager>>,
}

impl HandlerInner {
    pub(crate) fn new(flags: HandlerFlags, source_manager: Rc<RwLock<SourceManager>>) -> Self {
        Self {
            emitter: Emitter::new(flags, source_manager.clone()),
            // source_manager,
        }
    }
}

/// A Handler handles all Diagnostics that are to be emitted through the course of assembly.
/// Diagnostics are things such as warnings and errors.
pub struct Handler {
    /// The flags provided to this Handler specifying how it should behave
    flags: HandlerFlags,
    /// The InnerHandler that actually will do the emitting of diagnostics
    inner: Mutex<HandlerInner>,
}

impl Handler {
    /// Creates a new diagnostic Handler with the provided flags
    pub fn new(flags: HandlerFlags, source_manager: Rc<RwLock<SourceManager>>) -> Self {
        Self {
            flags,
            inner: Mutex::new(HandlerInner::new(flags, source_manager)),
        }
    }

    /// This registers a warning with this error Handler
    pub fn warn(&self, warning: Diagnostic) {
        // If we can't even emit them, don't even store them
        if self.flags.emit_warnings {
            if let Ok(inner) = self.inner.lock() {
                inner.emitter.emit_diagnostic(&warning);
            }
        }
    }

    /// This registers an error with this error Handler
    pub fn error(&self, error: Diagnostic) {
        if let Ok(inner) = self.inner.lock() {
            inner.emitter.emit_diagnostic(&error);
        }
    }
}

pub struct SourceManager {
    source_files: Vec<Rc<SourceFile>>,
}

impl SourceManager {
    pub fn new() -> Self {
        Self {
            source_files: Vec::new(),
        }
    }

    /// Adds a SourceFile to this SourceManager. The id field of the SourceFile is overwriten by
    /// this SourceManager so that it can be internally identified. Every other field of the
    /// SourceFile is left the same.
    pub fn add(&mut self, mut source_file: SourceFile) -> Result<u8, ()> {
        // Right now we require that the user only include a max of 256 files
        if self.source_files.len() < u8::MAX as usize {
            source_file.id = self.source_files.len() as u8;

            let id = source_file.id;

            self.source_files.push(Rc::new(source_file));

            Ok(id)
        } else {
            Err(())
        }
    }

    /// Gets a reference to a SourceFile by the SourceFile's id
    pub fn get_by_id(&self, id: usize) -> Option<Rc<SourceFile>> {
        // Because id == index of SourceFile as u8, we can just use it directly
        self.source_files.get(id).map(|sf| sf.clone())
    }
}

/// Represents a single KASM source file and associated data
pub struct SourceFile {
    /// The name of the source file. No path, only the form of: filename.ext
    pub name: String,
    /// The absolute path of this source file in the file system
    pub abs_path: Option<PathBuf>,
    /// The relative path of this source file to the place it was invoked
    pub rel_path: Option<PathBuf>,
    /// The actual source code of the file
    pub source: String,
    /// Each source file will be given a unique ID to be referred by inside of tokens
    pub id: u8,
}

impl SourceFile {
    pub fn new(
        name: String,
        abs_path: Option<PathBuf>,
        rel_path: Option<PathBuf>,
        source: String,
        id: u8,
    ) -> Self {
        Self {
            name,
            abs_path,
            rel_path,
            source,
            id,
        }
    }

    /// Gets the source location of a given span
    ///
    /// Note: This uses the span.start to determine the line and column
    ///
    /// The String returned as the path, is given as:
    ///
    /// src/main.kasm
    ///
    /// Or if the file has no path, it just returns the name of the file. So if it is from some
    /// kind of non-file input, then it is just displayed as <input>
    ///
    fn get_source_location(&self, span: &Span) -> (String, usize, usize) {
        let file_path = match &self.rel_path {
            Some(rel) => rel.to_str().unwrap().to_owned(),
            None => self.name.to_owned(),
        };

        let mut line_num = 1;
        let mut line_start_index = 0;

        // Loop through all characters until the span.start
        for (idx, c) in self.source.chars().take(span.start).enumerate() {
            if c == '\n' {
                line_num += 1;
                line_start_index = idx + 1;
            } else if c == '\t' {
                line_start_index -= 3;
            }
        }

        let col = span.start - line_start_index;

        (file_path, line_num, col)
    }

    /// Converts a Span into a Snippet by getting the source code for the Span
    pub fn span_to_snippet(&self, span: &Span) -> Snippet {
        let mut line_begin = span.start;
        let mut line_end = span.end;

        if self.source.chars().nth(span.start).unwrap() == '\n' {
            line_begin -= 1;
        }

        // Look for the beginning of the line this span is on
        while line_begin > 0 {
            if self.source.chars().nth(line_begin).unwrap() != '\n' {
                line_begin -= 1;
            } else {
                // Don't take the '\n' with us
                line_begin += 1;
                break;
            }
        }

        // Look for the end of the line this span is on
        while line_end < self.source.len() {
            if self.source.chars().nth(line_end).unwrap() != '\n' {
                line_end += 1;
            } else {
                break;
            }
        }

        let line = (&self.source[line_begin..line_end])
            .to_owned()
            .replace("\t", "    ")
            .replace("\n", " ");

        let before_start_col = span.start - line_begin;
        let mut start_col = before_start_col;

        for (col, c) in (&self.source[line_begin..line_end]).chars().enumerate() {
            if col < before_start_col && c == '\t' {
                start_col += 3;
            }
        }

        let end_col = start_col + (span.end - span.start);

        Snippet {
            line,
            start_col,
            end_col,
        }
    }
}

/// A Span is what Diagnostics use to display pieces of code. These can be turned into Snippets
/// which actually contain the source code that these snippets point to so that the Diagnostic can
/// be emitted.
#[derive(Debug, Copy, Clone)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub file: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, file: usize) -> Self {
        Self { start, end, file }
    }
}

#[derive(Debug, Clone)]
pub struct Snippet {
    pub line: String,
    pub start_col: usize,
    pub end_col: usize,
}

impl Snippet {
    pub fn as_slice(&self) -> &str {
        &self.line[self.start_col..self.end_col]
    }
}

impl Display for Snippet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_slice())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Level {
    Bug,
    Error,
    Warning,
    Note,
    Help,
    Cancelled,
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
            Level::Cancelled => {}
        }
        spec
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Level::Bug => "internal assembler error",
            Level::Error => "error",
            Level::Warning => "warning",
            Level::Note => "note",
            Level::Help => "help",
            Level::Cancelled => "cancelled",
        }
    }

    /// Returns true if this error level is considered fatal
    pub fn is_fatal(&self) -> bool {
        match self {
            Level::Bug => true,
            Level::Error => true,
            Level::Note => false,
            Level::Help => false,
            Level::Warning => false,
            Level::Cancelled => false,
        }
    }

    pub fn as_styled_string(&self) -> StyledString {
        match self {
            Level::Bug => StyledString::new(self.to_str().to_string(), Style::Level(*self)),
            Level::Error => StyledString::new(self.to_str().to_string(), Style::Level(*self)),
            Level::Note => StyledString::new(self.to_str().to_string(), Style::Level(*self)),
            Level::Help => StyledString::new(self.to_str().to_string(), Style::Level(*self)),
            Level::Warning => StyledString::new(self.to_str().to_string(), Style::Level(*self)),
            Level::Cancelled => unreachable!(),
        }
    }
}
