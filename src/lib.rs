#![allow(clippy::result_unit_err)]

use std::path::{Path, PathBuf};

use clap::{ArgAction, Parser};
use errors::SourceFile;
use kerbalobjects::ko::WritableKOFile;

pub mod errors;
pub mod session;

pub mod lexer;
pub mod output;
pub mod parser;
pub mod preprocessor;

use lexer::Token;
use session::Session;

use crate::{
    lexer::{phase0, Lexer, TokenKind},
    output::{generator::Generator, Verifier},
    parser::parse,
    preprocessor::executor::Executor,
};

pub static VERSION: &'_ str = env!("CARGO_PKG_VERSION");

/// Various configuration parameters for altering how the assembler acts
#[derive(Debug, Clone, Parser)]
pub struct Config {
    /// This value should be true when this assembler is being run in CLI mode, like in this crate
    /// itself. This causes it to emit errors to stdout, instead of just returning Err(())
    #[arg(skip = true)]
    pub emit_errors: bool,
    /// If warnings should be emitted during assembly
    #[arg(
        short = 'w',
        long = "no-warn",
        help = "Disables warnings from being displayed",
        action = ArgAction::SetFalse
    )]
    pub emit_warnings: bool,
    /// The "root directory" is usually the directory in which KASM was run, so that file paths can
    /// be expressed relative to the current location
    #[arg(skip = std::env::current_dir().expect("KASM run in directory that doesn't exist anymore"))]
    pub root_dir: PathBuf,
    /// If the preprocessor should be run or not. The benefit of not running it is that the
    /// assembly process will be faster without it
    #[arg(
        short = 'a',
        long = "no-preprocess",
        help = "Run the assembly process without running the preprocessor",
        action = ArgAction::SetFalse
    )]
    pub run_preprocessor: bool,
    /// If assembly should take place, or if the output file should be preprocessed source code.
    /// This can be useful for debugging or just generating code
    #[arg(
        short = 'p',
        long = "preprocess-only",
        help = "Instead of outputting an object file, emits KASM after the preprocessing step",
        conflicts_with("run_preprocessor")
    )]
    pub preprocess_only: bool,
    /// If specified, instead of the preprocessor looking at the current working directory for
    /// files to include, it will search the provided path
    #[arg(
        short = 'i',
        long = "include-path",
        help = "Specifies the include path for the assembler. Defaults to the current working directory"
    )]
    pub include_path: Option<PathBuf>,
    /// If specified, instead of the object file's "file" symbol being set to the name of the input
    /// file, it will be set to this provided value. This can be useful when creating a compiler
    /// with KASM as it allows you to use the source file's name and not the assembled file's name.
    #[arg(
        short = 'f',
        long = "file",
        help = "Adds a file symbol to the generated object file with the given name. Defaults to input file name"
    )]
    pub file_sym_name: Option<String>,
    /// If specified, instead of the default "Compiled with KASM {}", another comment will be
    /// placed inside of the produced object file. This is useful for setting messages for
    /// compilers that generate KASM
    #[arg(
        short = 'c',
        long = "comment",
        help = "Sets the comment field of the output object file to the value of this. Defaults to KASM and the current version",
        default_value_t = format!("Compiled by KASM {}", VERSION)
    )]
    pub comment: String,
}

/// Configuration parameters, but for exclusive use by a command line interface
#[derive(Debug, Clone, Parser)]
#[command(author, version, about = "Kerbal Assembler", long_about = None)]
pub struct CLIConfig {
    /// The input file path to load
    #[arg(value_name = "INPUT", help = "Sets the input file")]
    pub input_path: PathBuf,
    /// The output file path, which is now optional. If none is provided
    /// the file name will be the same as the input file, and the file extension
    /// is inferred by the assembler flags in Config
    #[arg(
        short = 'o',
        long = "output",
        value_name = "OUTPUT",
        help = "Sets the output path to use"
    )]
    pub output_path: Option<PathBuf>,
    #[command(flatten)]
    pub base_config: Config,
}

/// Represents the two possible types of output that KASM supports
pub enum AssemblyOutput {
    /// An assembled object file
    Object(Box<WritableKOFile>),
    /// Preprocessed source code
    Source(String),
}

/// Assemble a file given by a provided path
pub fn assemble_path(path: &Path, config: Config) -> Result<AssemblyOutput, ()> {
    let mut session = Session::new(config);

    // Check if we have been given a valid file
    if !session.is_file(path) {
        session
            .struct_error(format!("input `{}` is not a file", path.to_string_lossy()))
            .emit();

        return Err(());
    }

    // Read it
    match session.read_file(path) {
        Ok(_) => {}
        Err(e) => {
            session
                .struct_bug(format!(
                    "unable to read file `{}`: {}",
                    path.to_string_lossy(),
                    e
                ))
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
    if session.config().preprocess_only {
        let output = generate_preprocessed(tokens, &session);

        return Ok(AssemblyOutput::Source(output));
    }

    let parser = parse::Parser::new(tokens, &session);

    let (parsed_functions, label_manager, symbol_manager) = parser.parse()?;

    let verifier = Verifier::new(parsed_functions, &session, &label_manager, &symbol_manager);

    let verified_functions = verifier.verify()?;

    let generator = Generator::new(&session, &symbol_manager);

    let kofile = generator.generate(verified_functions)?;

    Ok(AssemblyOutput::Object(Box::new(kofile)))
}

// Generates preprocessed source output
fn generate_preprocessed(tokens: Vec<Token>, session: &Session) -> String {
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

    output
}
