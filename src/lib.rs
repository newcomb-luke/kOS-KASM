use clap::ArgMatches;
use std::{error::Error, path::Path, fs};

mod lexer;
pub use lexer::{Lexer, Token};

mod preprocessor;
pub use preprocessor::{
    BinOp, DefinitionTable, ExpNode, ExpressionEvaluator, ExpressionParser, UnOp,
    Value, ValueType, Preprocessor
};

mod parser;
pub use parser::{Instruction, Label, TextEntry};

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

    let main_contents = fs::read_to_string(&config.file_path)?;

    let main_tokens = Lexer::lex(&main_contents)?;

    for token in &main_tokens {
        println!("{:?}", token);
    }

    let processed_tokens = preprocessor.process(main_tokens)?;

    println!("Preprocessed:");

    for token in &processed_tokens {
        println!("{:?}", token);
    }

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
