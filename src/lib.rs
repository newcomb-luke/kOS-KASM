#![allow(clippy::result_unit_err)]

use std::path::PathBuf;

use errors::SourceFile;
use kerbalobjects::kofile::KOFile;

pub mod errors;
pub mod session;

pub mod lexer;
pub mod output;
pub mod parser;
pub mod preprocessor;

use session::Session;

use crate::{
    lexer::{phase0, Lexer, TokenKind},
    output::Verifier,
    parser::parse,
    preprocessor::executor::Executor,
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
    /// If assembly should take place, or if the output file should be preprocessed source code.
    /// This can be useful for debugging or just generating code
    pub output_preprocessed: bool,
    /// If specified, instead of the preprocessor looking at the current working directory for
    /// files to include, it will search the provided path
    pub include_path: Option<String>,
    /// If specified, instead of the object file's "file" symbol being set to the name of the input
    /// file, it will be set to this provided value. This can be useful when creating a compiler
    /// with KASM as it allows you to use the source file's name and not the assembled file's name.
    pub file_sym_name: Option<String>,
}

/// Represents the two possible types of output that KASM supports
pub enum AssemblyOutput {
    /// An assembled object file
    Object(KOFile),
    /// Preprocessed source code
    Source(String),
}

/// Assemble a file given by a provided path
pub fn assemble_path(path: String, config: Config) -> Result<AssemblyOutput, ()> {
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
        Err(e) => {
            session
                .struct_bug(format!("unable to read file `{}`: {}", &path, e))
                .emit();

            return Err(());
        }
    };

    assemble(session)
}

/// Assemble a file given by a string
pub fn assemble_string(source: String, config: Config) -> Result<AssemblyOutput, ()> {
    let mut session = Session::new(config);

    // Create a SourceFile but with some dummy values
    let source_file = SourceFile::new("<input>".to_owned(), None, None, source, 0);

    session.add_file(source_file);

    assemble(session)
}

// The core of the assembler. The actual function that runs everything else
// This should be called with a session that already has the primary source file read
fn assemble(mut session: Session) -> Result<AssemblyOutput, ()> {
    let primary_file = session.get_file(0).unwrap();

    // Create the lexer
    let lexer = Lexer::new(&primary_file.source, 0, &session);

    // Lex the tokens, if they are all valid
    let mut tokens = lexer.lex()?;

    // Replace comments and line continuations
    phase0(&mut tokens, &session)?;

    // If we should run the preprocessor
    if session.config().run_preprocessor {
        let preprocessor_parser = preprocessor::parser::Parser::new(tokens, &session);

        let nodes = preprocessor_parser.parse()?;

        let executor = Executor::new(&mut session);

        tokens = executor.execute(nodes)?;
    }

    // If we should output the preprocessed tokens instead of assembling
    if session.config().output_preprocessed {
        let mut output = String::new();

        for token in tokens {
            let str_rep = match token.kind {
                TokenKind::Newline => "\n",
                TokenKind::OperatorMinus => "-",
                TokenKind::OperatorPlus => "+",
                TokenKind::OperatorCompliment => "~",
                TokenKind::OperatorMultiply => "*",
                TokenKind::OperatorDivide => "/",
                TokenKind::OperatorMod => "%",
                TokenKind::OperatorAnd => "&&",
                TokenKind::OperatorOr => "||",
                TokenKind::OperatorEquals => "==",
                TokenKind::OperatorNotEquals => "!=",
                TokenKind::OperatorNegate => "!",
                TokenKind::OperatorGreaterThan => ">",
                TokenKind::OperatorLessThan => "<",
                TokenKind::OperatorGreaterEquals => ">=",
                TokenKind::OperatorLessEquals => "<=",
                TokenKind::SymbolLeftParen => "(",
                TokenKind::SymbolRightParen => ")",
                TokenKind::SymbolComma => ",",
                TokenKind::SymbolHash => "#",
                TokenKind::SymbolAt => "@",
                TokenKind::SymbolAnd => "&",
                TokenKind::LiteralTrue => "true",
                TokenKind::LiteralFalse => "false",
                TokenKind::Backslash => "\\",
                TokenKind::KeywordSection => ".section",
                TokenKind::KeywordText => ".text",
                TokenKind::KeywordData => ".data",
                TokenKind::TypeI8 => ".i8",
                TokenKind::TypeI16 => ".i16",
                TokenKind::TypeI32 => ".i32",
                TokenKind::TypeI32V => ".i32v",
                TokenKind::TypeF64 => ".f64",
                TokenKind::TypeF64V => ".f64v",
                TokenKind::TypeS => ".s",
                TokenKind::TypeSV => ".sv",
                TokenKind::TypeB => ".b",
                TokenKind::TypeBV => ".bv",
                TokenKind::DirectiveDefine => ".define",
                TokenKind::DirectiveMacro => ".macro",
                TokenKind::DirectiveEndmacro => ".endmacro",
                TokenKind::DirectiveRepeat => ".rep",
                TokenKind::DirectiveEndRepeat => ".endrep",
                TokenKind::DirectiveInclude => ".include",
                TokenKind::DirectiveExtern => ".extern",
                TokenKind::DirectiveGlobal => ".global",
                TokenKind::DirectiveLocal => ".local",
                TokenKind::DirectiveLine => ".line",
                TokenKind::DirectiveType => ".type",
                TokenKind::DirectiveValue => ".value",
                TokenKind::DirectiveUndef => ".undef",
                TokenKind::DirectiveUnmacro => ".unmacro",
                TokenKind::DirectiveFunc => ".func",
                TokenKind::DirectiveIf => ".if",
                TokenKind::DirectiveIfNot => ".ifn",
                TokenKind::DirectiveIfDef => ".ifdef",
                TokenKind::DirectiveIfNotDef => ".ifndef",
                TokenKind::DirectiveElseIf => ".elif",
                TokenKind::DirectiveElseIfNot => ".elifn",
                TokenKind::DirectiveElseIfDef => ".elifdef",
                TokenKind::DirectiveElseIfNotDef => ".elifndef",
                TokenKind::DirectiveElse => ".else",
                TokenKind::DirectiveEndIf => ".endif",
                TokenKind::InnerLabelReference
                | TokenKind::InnerLabel
                | TokenKind::Identifier
                | TokenKind::Label
                | TokenKind::Whitespace
                | TokenKind::LiteralInteger
                | TokenKind::LiteralFloat
                | TokenKind::LiteralHex
                | TokenKind::LiteralBinary
                | TokenKind::LiteralString
                | TokenKind::Comment
                | TokenKind::Error
                | TokenKind::JunkFloatError => "",
            };

            if !str_rep.is_empty() {
                output.push_str(str_rep);
            } else {
                let snippet = session.span_to_snippet(&token.as_span());
                let token_str = snippet.as_slice();

                output.push_str(token_str);
            }
        }

        return Ok(AssemblyOutput::Source(output));
    }

    let parser = parse::Parser::new(tokens, &session);

    let (parsed_functions, label_manager, symbol_manager) = parser.parse()?;

    let verifier = Verifier::new(parsed_functions, &session, &label_manager, &symbol_manager);

    let verified_functions = verifier.verify()?;

    todo!();
}
