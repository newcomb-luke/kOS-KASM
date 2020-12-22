use clap::ArgMatches;
use std::{error::Error, path::Path, fs, fs::File};
use std::io::Write;

mod lexer;
pub use lexer::{Lexer, Token, TokenType, TokenData};

mod preprocessor;
pub use preprocessor::{
    BinOp, DefinitionTable, ExpNode, ExpressionEvaluator, ExpressionParser, UnOp,
    Value, ValueType, Definition, Macro, Preprocessor, MacroTable, LabelManager, PreprocessorSettings, Label, LabelType, LabelInfo, LabelValue
};

mod parser;
pub use parser::{Instruction, OperandType, pass1, pass2};

mod output;
pub use output::{tokens_to_text};

use kerbalobjects::{KOFileWriter};

pub static VERSION: &'static str = "0.9.0";

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

        // Replace the file extension of .kasm with .ko
        output_path.replace_range((output_path.len() - 4).., "ko");
    } else if !output_path.ends_with(".ko") {
        output_path.push_str(".ko");
    }

    let mut include_path = config.include_path.clone();

    if include_path.is_empty() {
       include_path = String::from(Path::new(&config.file_path).parent().unwrap().canonicalize().unwrap().to_str().unwrap());
    } else {
        if !Path::new(&include_path).is_dir() {
            return Err("Include path must be a directory.".into());
        }
    }

    if config.debug {
        println!("Outputting to: {}", output_path);
        println!("Include path: {}", include_path);
    }

    let mut preprocessor = Preprocessor::new(include_path);
    let mut definition_table = DefinitionTable::new();
    let mut macro_table = MacroTable::new();
    let mut label_manager = LabelManager::new();
    let mut input_files = InputFiles::new();
    input_files.add_file("main");

    let settings = PreprocessorSettings { expand_definitions: true, expand_macros: true };

    let main_contents = fs::read_to_string(&config.file_path)?;

    let mut lexer = Lexer::new();

    let main_tokens = lexer.lex(&main_contents, "main", 0)?;

    let processed_tokens = preprocessor.process(&settings, main_tokens, &mut definition_table, &mut macro_table, &mut label_manager, &mut input_files)?;

    let preprocessed = tokens_to_text(&processed_tokens);

    let mut pre_file = File::create("preprocessed.kasm")?;

    pre_file.write_all(&preprocessed.as_bytes())?;

    let pass1_tokens = pass1(&processed_tokens, &mut label_manager)?;

    let mut writer = KOFileWriter::new(&output_path);

    let mut kofile = pass2(&pass1_tokens, &mut label_manager)?;

    kofile.write(&mut writer)?;

    writer.write_to_file()?;

    Ok(())
}

pub struct CLIConfig {
    pub file_path: String,
    pub output_path_value: String,
    pub debug: bool,
    pub include_path: String,
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
            debug: matches.is_present("debug"),
            include_path: if matches.is_present("include_path") {
                String::from(matches.value_of("include_path").unwrap())
            } else {
                String::new()
            },
        }
    }
}

pub struct InputFiles {
    files: Vec<String>
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