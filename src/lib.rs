use clap::ArgMatches;
use std::error::Error;

mod lexer;
pub use lexer::{Lexer, Token};

mod parser;
pub use parser::{ExpressionParser, ExpNode, Value, DefinitionTable, Definition};

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

    if config.debug {
        println!("Outputting to: {}", output_path);
    }

    // let tokens = Lexer::lex(
    //     r#"
    //     .define PI 3.14

    //     .define PUSH2 push 2

    //     .define a(b)    1 + b(x)

    //     .macro somemacro 1
    //         push 1
    //         push &1
    //         add
    //     .endmacro

    //     push 0xffff
    //     stoe "$z"

    //     push 0b1111_0010
    //     stoe "$z"

    //     .include "somefilename.extensions"

    //     .define YOURMOM  2 + 2 > 5 \
    //      || 100 / 20 % 5 == 1

    //      loop:
    //         INC  "$x"
    //         stoe "$x"
    //         .inner:
    //             push YOURMOM
    //             stoe "$y"
    // "#,
    // )?;

    let tokens = Lexer::lex("0xffffff + 2 * 3 || false || !(NUM_SWORDS / 2)")?;

    for token in &tokens {
        println!("{:?}", token);
    }

    let exp = ExpressionParser::parse_expression(&mut tokens.iter().peekable())?;

    println!("{:#?}", exp);

    let mut defTable = DefinitionTable::new();

    defTable.def("NUM_SWORDS", Definition::Constant(ExpNode::Constant(Value::Int(20))));

    

    Ok(())
}

pub struct CLIConfig {
    pub file_path: String,
    pub output_path_value: String,
    pub debug: bool,
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
        }
    }
}
