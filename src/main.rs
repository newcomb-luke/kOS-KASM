use clap::{App, Arg};
use kasm::Config;
use std::process;

use kasm::assemble_path;

// use kasm::{run, CLIConfig};

fn main() {
    let matches = App::new("Kerbal Assembler")
        .version(kasm::VERSION)
        .author("Luke Newcomb")
        .about("Assembles KerbalAssembly files into KerbalObject files to be linked.")
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("disable_warnings")
                .help("Disables warnings from being displayed")
                .short("w")
                .long("no-warn"),
        )
        .arg(
            Arg::with_name("output_path")
                .help("Sets the output file to use")
                .short("o")
                .long("output")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("include_path")
                .help("Specifies the include path for the assembler. Defaults to the current working directory.")
                .short("i")
                .long("include-path")
                .require_equals(true)
                .takes_value(true)
        )
        .arg(
            Arg::with_name("preprocess_only")
                .help("Instead of outputting an object file, emits KASM after the preprocessing step.")
                .short("p")
                .long("preprocess-only")
        )
        .arg(
            Arg::with_name("no_preprocess")
                .help("Run the assembly process without running the preprocessor.")
                .short("a")
                .long("no-preprocess")
                .conflicts_with("preprocess_only")
            )
        .arg(
            Arg::with_name("comment")
                .help("Sets the comment field of the output object file to the value of this. Defaults to KASM and the current version.")
                .short("c")
                .long("comment")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("file")
                .help("Adds a file symbol to the generated object file with the given name. Defaults to input file name.")
                .short("f")
                .long("file")
                .takes_value(true)
        )
        .get_matches();

    // This is a required argument, so it won't panic
    let path = matches.value_of("INPUT").unwrap().to_string();

    // Do conversion for the arguments
    let emit_warnings = !matches.is_present("disable_warnings");

    let run_preprocessor = !matches.is_present("no_preprocess");

    // Get the directory this was run from
    let root_dir =
        std::env::current_dir().expect("KASM run in directory that doesn't exist anymore");

    let config = Config {
        is_cli: true, // Always true, this is main.rs!
        emit_warnings,
        root_dir,
        run_preprocessor,
    };

    if let Err(_) = assemble_path(path, config) {
        process::exit(1);
    }
}
