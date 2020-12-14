use clap::ArgMatches;
use std::{error::Error, path::Path, fs, fs::File};
use std::io::Write;

mod lexer;
pub use lexer::{Lexer, Token, TokenType, TokenData};

mod preprocessor;
pub use preprocessor::{
    BinOp, DefinitionTable, ExpNode, ExpressionEvaluator, ExpressionParser, UnOp,
    Value, ValueType, Definition, Macro, Preprocessor, MacroTable, SymbolTable, PreprocessorSettings
};

mod parser;
pub use parser::{Instruction, Label};

mod output;
pub use output::{tokens_to_text};

pub static VERSION: &'static str = "0.1.0";

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
    let mut symbol_table = SymbolTable::new();
    let mut input_files = InputFiles::new();
    input_files.add_file("main");

    let settings = PreprocessorSettings { expand_definitions: true, expand_macros: true };

    let main_contents = fs::read_to_string(&config.file_path)?;

    let mut lexer = Lexer::new();

    let main_tokens = lexer.lex(&main_contents, "main", 0)?;

    for token in &main_tokens {
        println!("{}", token.as_str());
    }

    let processed_tokens = preprocessor.process(&settings, main_tokens, &mut definition_table, &mut macro_table, &mut symbol_table, &mut input_files)?;

    println!("---------------------------------------------------------------------");
    println!("Preprocessed:\n");

    for token in &processed_tokens {
        println!("{}", token.as_str());
    }

    let preprocessed = tokens_to_text(&processed_tokens);

    let mut pre_file = File::create("preprocessed.kasm")?;

    pre_file.write_all(&preprocessed.as_bytes())?;

    // let exp = ExpressionParser::parse_expression(&mut tokens.iter().peekable())?.unwrap();

    // let mut def_table = DefinitionTable::new();

    // def_table.def(
    //     "NUM_SWORDS",
    //     Definition::Constant(ExpNode::Constant(Value::Int(2))),
    // );

    // def_table.def(
    //     "NUM_HOLDERS",
    //     Definition::Constant(ExpNode::Constant(Value::Int(20))),
    // );

    // let result = ExpressionEvaluator::evaluate(&mut def_table, &exp)?;

    // println!("{:?}", result);

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