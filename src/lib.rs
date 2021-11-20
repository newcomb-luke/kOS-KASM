#![allow(clippy::result_unit_err)]

use std::path::PathBuf;

use errors::SourceFile;
use kerbalobjects::kofile::KOFile;

pub mod errors;
pub mod session;

pub mod lexer;
pub mod preprocessor;

use session::Session;

use crate::{
    lexer::{phase0, Lexer},
    preprocessor::{executor::Executor, parser::Parser},
};

pub static VERSION: &'_ str = env!("CARGO_PKG_VERSION");

/// Various configuration parameters for altering how the assembler acts
pub struct Config {
    /// This value should be true when this assembler is being run in CLI mode, like in this crate
    /// itself. This causes it to emit errors to stdout, instead of just returning Err(())
    pub is_cli: bool,
    /// If warnings should be emitted during assembly
    pub emit_warnings: bool,
    /// The "root directory" is usually the directory in which KASM was run, so that file paths can
    /// be expressed relative to the current location
    pub root_dir: PathBuf,
    /// If the preprocessor should be run or not. The benefit of not running it is that the
    /// assembly process will be faster without it
    pub run_preprocessor: bool,
}

/// Assemble a file given by a provided path
pub fn assemble_path(path: String, config: Config) -> Result<KOFile, ()> {
    let mut session = Session::new(config);

    // Check if we have been given a valid file
    if !session.is_file(&path) {
        session
            .struct_error(format!("input `{}` is not a file", &path))
            .emit();

        return Err(());
    }

    // Read it
    match session.read_file(&path) {
        Ok(_) => {}
        Err(_) => {
            session
                .struct_bug(format!("unable to read file `{}`", &path))
                .emit();

            return Err(());
        }
    };

    assemble(session)
}

/// Assemble a file given by a string
pub fn assemble_string(source: String, config: Config) -> Result<KOFile, ()> {
    let mut session = Session::new(config);

    // Create a SourceFile but with some dummy values
    let source_file = SourceFile::new("<input>".to_owned(), None, None, source, 0);

    session.add_file(source_file);

    assemble(session)
}

// The core of the assembler. The actual function that runs everything else
// This should be called with a session that already has the primary source file read
fn assemble(session: Session) -> Result<KOFile, ()> {
    let primary_file = session.get_file(0).unwrap();

    // Create the lexer
    let lexer = Lexer::new(&primary_file.source, session);

    // Lex the tokens, if they are all valid
    let (mut tokens, mut session) = lexer.lex()?;

    // Replace comments and line continuations
    phase0(&mut tokens, &mut session)?;

    // If we should run the preprocessor
    if session.config().run_preprocessor {
        let mut preprocessor_parser = Parser::new(tokens, session);

        let nodes = preprocessor_parser.parse()?;

        let mut executor = Executor::new(nodes);

        tokens = executor.execute()?;
    }

    todo!();
}
