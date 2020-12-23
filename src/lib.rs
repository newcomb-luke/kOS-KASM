use clap::ArgMatches;
use std::env;
use std::io::Write;
use std::{error::Error, fs, fs::File, path::Path};

mod lexer;
pub use lexer::{Lexer, Token, TokenData, TokenType};

mod preprocessor;
pub use preprocessor::{
    BinOp, Definition, DefinitionTable, ExpNode, ExpressionEvaluator, ExpressionParser, Label,
    LabelInfo, LabelManager, LabelType, LabelValue, Macro, MacroTable, Preprocessor,
    PreprocessorSettings, UnOp, Value, ValueType,
};

mod parser;
pub use parser::{pass1, pass2, Instruction, OperandType};

mod output;
pub use output::tokens_to_text;

use kerbalobjects::{KOFileWriter, StringTable};

pub static VERSION: &'static str = "0.9.5";

pub fn run(config: &CLIConfig) -> Result<(), Box<dyn Error>> {
    if !config.file_path.ends_with(".kasm") {
        return Err(format!(
            "Input file must be a KASM file. Found: {}",
            config.file_path
        )
        .into());
    }

    let mut output_path = config.output_path_value.clone();

    // If the output path was not specified
    if output_path.is_empty() {
        // Create a new string the same as file_path
        output_path = config.file_path.clone();

        // Check if we are only preprocessing or not
        if config.preprocess_only {
            // Replce the extension with _pre.kasm
            output_path.replace_range((output_path.len() - 4).., "_pre.kasm");
        } else {
            // Replace the file extension of .kasm with .ko
            output_path.replace_range((output_path.len() - 4).., "ko");
        }
    } else {
        // Check if we are only preprocessing or not
        if config.preprocess_only {
            if !output_path.ends_with(".kasm") {
                output_path.push_str(".kasm");
            }
        } else {
            if !output_path.ends_with(".ko") {
                output_path.push_str(".ko");
            }
        }
    }

    let mut include_path = config.include_path.clone();

    // If no include path has been specified
    if include_path.is_empty() {
        // Get it from the current working directory
        let cwd = env::current_dir()?;

        include_path = String::from(cwd.as_path().to_str().unwrap());
    } else {
        if !Path::new(&include_path).is_dir() {
            return Err("Include path must be a directory.".into());
        }
    }

    // Create all the variables required to perform assembly
    let mut preprocessor = Preprocessor::new(include_path);
    let mut definition_table = DefinitionTable::new();
    let mut macro_table = MacroTable::new();
    let mut label_manager = LabelManager::new();
    let mut input_files = InputFiles::new();
    let settings = PreprocessorSettings {
        expand_definitions: true,
        expand_macros: true,
    };

    input_files.add_file("main");

    // Read the input file
    let main_contents = fs::read_to_string(&config.file_path)?;

    // Create out lexer and lex the main file's tokens
    let mut lexer = Lexer::new();
    let main_tokens = lexer.lex(&main_contents, "main", 0)?;

    // Run preprocessor
    let processed_tokens = preprocessor.process(
        &settings,
        main_tokens,
        &mut definition_table,
        &mut macro_table,
        &mut label_manager,
        &mut input_files,
    )?;

    // If we are just output the preprocessed only
    if config.preprocess_only {
        // If we are, just output that
        let preprocessed = tokens_to_text(&processed_tokens);
        let mut pre_file = File::create(&output_path)?;
        pre_file.write_all(&preprocessed.as_bytes())?;
    }
    // If not
    else {
        // Run pass 1
        let pass1_tokens = pass1(&processed_tokens, &mut label_manager)?;

        for label in label_manager.as_vec().iter() {
            println!("Label: {}", label.as_str());
        }

        println!("-----------------------------");

        // Run pass 2
        let mut kofile = pass2(&pass1_tokens, &mut label_manager)?;

        // We will always add a comment
        let comment_str;

        // Check if there is a comment to add
        if !config.comment.is_empty() {
            comment_str = config.comment.to_owned();
        }
        // If there isn't, then write that this file was created using this assembler
        else {
            comment_str = format!("Assembled by KASM v{}", VERSION);
        }

        let mut comment_strtab = StringTable::new(".comment");

        // Add the comment as the first and only string
        comment_strtab.add(&comment_str);

        // Add the comment section
        kofile.add_string_table(comment_strtab);

        // Create a KO file writer
        let mut writer = KOFileWriter::new(&output_path);

        // Actually write the file to disk
        kofile.write(&mut writer)?;
        writer.write_to_file()?;
    }

    Ok(())
}

pub struct CLIConfig {
    pub file_path: String,
    pub output_path_value: String,
    pub preprocess_only: bool,
    pub include_path: String,
    pub comment: String,
}

impl CLIConfig {
    pub fn new(matches: ArgMatches) -> CLIConfig {
        CLIConfig {
            file_path: String::from(matches.value_of("INPUT").unwrap()),
            output_path_value: if matches.is_present("output_path") {
                String::from(matches.value_of("output_path").unwrap())
            } else {
                String::new()
            },
            preprocess_only: matches.is_present("preprocess_only"),
            include_path: if matches.is_present("include_path") {
                String::from(matches.value_of("include_path").unwrap())
            } else {
                String::new()
            },
            comment: if matches.is_present("comment") {
                String::from(matches.value_of("comment").unwrap())
            } else {
                String::new()
            },
        }
    }
}

pub struct InputFiles {
    files: Vec<String>,
}

impl InputFiles {
    pub fn new() -> InputFiles {
        InputFiles { files: Vec::new() }
    }

    /// Adds the file to the internal vector, and returns the file's id
    pub fn add_file(&mut self, file: &str) -> usize {
        self.files.push(String::from(file));

        self.files.len()
    }

    pub fn get_from_id(&self, id: usize) -> String {
        self.files.get(id).unwrap().to_owned()
    }
}
