use clap::ArgMatches;
use kerbalobjects::kofile::sections::{SectionHeader, StringTable};
use kerbalobjects::kofile::symbols::KOSymbol;
use output::preprocessed::tokens_to_text;
use std::io::Write;
use std::{
    env,
    fmt::{Display, Formatter},
};
use std::{error::Error, fs, fs::File, path::Path};

pub mod lexer;
use lexer::*;

pub mod preprocessor;
use preprocessor::*;

pub mod parser;
use parser::*;

pub mod output;
use output::generator::Generator;

use kerbalobjects::{kofile::*, ToBytes};

pub static VERSION: &'static str = "0.10.12";

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

    let mut include_path = config.include_path.clone();

    // If no include path has been specified
    if include_path == "!" {
        // Get it from the current working directory
        let cwd = env::current_dir()?;

        include_path = String::from(cwd.as_path().to_str().unwrap());
    } else {
        if !Path::new(&include_path).is_dir() {
            return Err(KASMError::IncludePathDirectoryError.into());
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

            let symstrtab = kofile.str_tab_by_name(".symstrtab").unwrap();

            let file_name_idx = symstrtab.add(file_name);
            let mut file_sym =
                KOSymbol::new(0, 0, symbols::SymBind::Global, symbols::SymType::File, 0);
            file_sym.set_name_idx(file_name_idx);

            let sym_section = kofile.sym_tab_by_name(".symtab").unwrap();

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

#[derive(Debug)]
pub enum KASMError {
    IncludePathDirectoryError,
}

impl Error for KASMError {}

impl Display for KASMError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            KASMError::IncludePathDirectoryError => {
                write!(f, "Include path must be a directory.")
            }
        }
    }
}
