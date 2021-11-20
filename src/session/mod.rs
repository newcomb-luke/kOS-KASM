use std::{path::PathBuf, rc::Rc, sync::RwLock};

use crate::{
    errors::{
        DiagnosticBuilder, Handler, HandlerFlags, Level, Snippet, SourceFile, SourceManager, Span,
    },
    Config,
};

pub struct Session {
    source_manager: Rc<RwLock<SourceManager>>,
    config: Config,
    handler: Handler,
    num_files: usize,
}

impl Session {
    pub fn new(config: Config) -> Self {
        let flags = HandlerFlags {
            colored_output: Self::colored_output(),
            emit_warnings: config.emit_warnings,
            quiet: !config.is_cli,
        };

        let source_manager = Rc::new(RwLock::new(SourceManager::new()));

        Self {
            source_manager: source_manager.clone(),
            config,
            handler: Handler::new(flags, source_manager),
            num_files: 0,
        }
    }

    pub fn span_to_snippet(&self, span: &Span) -> Snippet {
        self.source_manager
            .read()
            .unwrap()
            .get_by_id(span.file)
            .unwrap()
            .span_to_snippet(span)
    }

    pub fn get_file(&self, file_id: usize) -> Option<Rc<SourceFile>> {
        self.source_manager.read().unwrap().get_by_id(file_id)
    }

    pub fn is_file(&self, path: &str) -> bool {
        PathBuf::from(path).is_file()
    }

    pub fn at_file_max(&self) -> bool {
        self.num_files > u8::MAX as usize
    }

    pub fn read_file(&mut self, path: &str) -> std::io::Result<u8> {
        let path_buf = PathBuf::from(&path);

        // This should be fine, given that we _should have_ already checked that this is a file
        let file_name_os = path_buf.file_name().unwrap();

        // _And_ since we got this path from a String, it should still be a valid string
        let file_name = file_name_os.to_str().unwrap().to_owned();

        let abs_path = std::fs::canonicalize(&path_buf)?;

        let rel_path = pathdiff::diff_paths(&abs_path, &self.config.root_dir).unwrap();

        let source = std::fs::read_to_string(&path)?;

        // The file id will be replaced by the source manager anyway
        let source_file = SourceFile::new(file_name, Some(abs_path), Some(rel_path), source, 0);

        self.num_files += 1;

        Ok(self
            .source_manager
            .write()
            .unwrap()
            .add(source_file)
            .unwrap())
    }

    /// This function should ONLY be used for debugging/tests
    pub fn add_file(&mut self, source_file: SourceFile) {
        self.source_manager
            .write()
            .unwrap()
            .add(source_file)
            .unwrap();
    }

    pub fn struct_span_warn(&self, span: Span, message: String) -> DiagnosticBuilder<'_> {
        let mut db = DiagnosticBuilder::new(&self.handler, Level::Warning, message);

        db.set_primary_span(span);

        db
    }

    pub fn struct_span_error(&self, span: Span, message: String) -> DiagnosticBuilder<'_> {
        let mut db = DiagnosticBuilder::new(&self.handler, Level::Error, message);

        db.set_primary_span(span);

        db
    }

    pub fn struct_bug(&self, message: String) -> DiagnosticBuilder<'_> {
        DiagnosticBuilder::new(&self.handler, Level::Bug, message)
    }

    pub fn struct_error(&self, message: String) -> DiagnosticBuilder<'_> {
        DiagnosticBuilder::new(&self.handler, Level::Error, message)
    }

    pub fn struct_warn(&self, message: String) -> DiagnosticBuilder<'_> {
        DiagnosticBuilder::new(&self.handler, Level::Warning, message)
    }

    // Returns true if error output should be colored, false if not
    fn colored_output() -> bool {
        atty::is(atty::Stream::Stderr)
    }

    pub fn config(&self) -> &Config {
        &self.config
    }
}
