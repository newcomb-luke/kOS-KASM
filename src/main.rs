use clap::{App, Arg};
use std::process;

use kasm::{run, CLIConfig};

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

    let config = CLIConfig::new(matches);

    if let Err(e) = run(&config) {
        eprintln!("Application error: {}", e);

        process::exit(1);
    }
}
