use clap::ArgMatches;
use errors::SourceFile;
use lexer::check_errors;
use lexer::token::Token;
use preprocessor::phase0::phase0;
use std::env;
use std::{error::Error, path::Path};
use std::{fs, vec};

use crate::preprocessor::phase0::phase1;

pub mod lexer;

pub mod errors;

/*
pub mod preprocessor;
use preprocessor::*;

pub mod parser;
use parser::*;

pub mod output;
use output::generator::Generator;
*/

pub mod preprocessor;

pub mod output;

pub static VERSION: &'static str = env!("CARGO_PKG_VERSION");

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
    if output_path == "!" {
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

    let mut _include_path = config.include_path.clone();

    // If no include path has been specified
    if _include_path == "!" {
        // Get it from the current working directory
        let cwd = env::current_dir()?;

        _include_path = String::from(cwd.as_path().to_str().unwrap());
    } else {
        if !Path::new(&_include_path).is_dir() {
            return Err("Lol".into());
        }
    }

    let input_path = Path::new(&config.file_path);

    let input_file_name_os = input_path.file_name().ok_or("Invalid path provided")?;
    let input_file_name = input_file_name_os
        .to_owned()
        .into_string()
        .map_err(|_| "Invalid path provided")?;

    /*
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
    */

    // input_files.add_file("main");

    // Read the input file
    let main_source = fs::read_to_string(&config.file_path)?;

    let source_file = SourceFile::new(input_file_name, main_source);

    let source_files = vec![source_file];

    let mut tokens: Vec<Token> = lexer::tokenize(source_files.get(0).unwrap().source()).collect();

    if let Err(errors) = check_errors(&tokens) {
        for error in errors {
            error.emit(&source_files)?;
        }

        return Err("".into());
    }

    // Run phase 0 of the assembly
    if let Err(error) = phase0(&mut tokens) {
        error.emit(&source_files)?;

        return Err("".into());
    }

    // Run phase 1 of the assembly
    tokens = phase1(tokens);

    println!("Lexing complete");

    /*
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
            // Run the parser
            let output = Parser::parse(processed_tokens, &mut label_manager)?;

            let mut kofile = Generator::generate(output, &mut label_manager)?;

            // Check if an empty comment was specified
            if !config.comment.is_empty() {
                // If it isn't, then check if any comment was specified
                let comment_str = if config.comment != "!" {
                    config.comment.to_owned()
                } else {
                    format!("Assembled by KASM v{}", VERSION)
                };

                let comment_strtab_name_idx = kofile.add_shstr(".comment");

                let comment_strtab_sh =
                    SectionHeader::new(comment_strtab_name_idx, sections::SectionKind::StrTab);

                let comment_strtab_idx = kofile.add_header(comment_strtab_sh);

                let mut comment_strtab = StringTable::new(0, comment_strtab_idx);

                // Add the comment as the first and only string
                comment_strtab.add(&comment_str);
                // Add the comment section
                kofile.add_str_tab(comment_strtab);
            }

            // Check if a non-empty file name has been specified
            if !config.file.is_empty() {
                // We need to get the file name we will put as the FILE symbol in the object file
                let file_name = if config.file == "!" {
                    Path::new(&config.file_path)
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                } else {
                    &config.file
                };

                let symstrtab = kofile.str_tab_by_name_mut("symstrtab").unwrap();

                let file_name_idx = symstrtab.add(file_name);
                let file_sym = KOSymbol::new(
                    file_name_idx,
                    0,
                    0,
                    symbols::SymBind::Global,
                    symbols::SymType::File,
                    0,
                );

                let sym_section = kofile.sym_tab_by_name_mut(".symtab").unwrap();

                // Add it
                sym_section.add(file_sym);
            }

            kofile.update_headers()?;

            let mut file_buffer = Vec::with_capacity(2048);

            kofile.to_bytes(&mut file_buffer);

            // Actually write the file to disk
            let mut file =
                std::fs::File::create(output_path).expect("Output file could not be created");

            file.write_all(file_buffer.as_slice())
                .expect("Output file could not be written to.");
        }
    */

    Ok(())
}

pub struct CLIConfig {
    pub file_path: String,
    pub output_path_value: String,
    pub preprocess_only: bool,
    pub include_path: String,
    pub comment: String,
    pub file: String,
}

impl CLIConfig {
    pub fn new(matches: ArgMatches) -> CLIConfig {
        CLIConfig {
            file_path: String::from(matches.value_of("INPUT").unwrap()),
            output_path_value: String::from(matches.value_of("output_path").unwrap_or("!")),
            preprocess_only: matches.is_present("preprocess_only"),
            include_path: String::from(matches.value_of("include_path").unwrap_or("!")),
            comment: String::from(matches.value_of("comment").unwrap_or("!")),
            file: String::from(matches.value_of("file").unwrap_or("!")),
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
