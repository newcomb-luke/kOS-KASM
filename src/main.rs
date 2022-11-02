use clap::Parser;
use kasm::{AssemblyOutput, CLIConfig};
use std::fs::File;
use std::path::Path;
use std::{io::Write, process};

use kasm::assemble_path;

fn main() {
    let config: CLIConfig = CLIConfig::parse();

    if let Ok(output) = assemble_path(&config.input_path, config.base_config) {
        match output {
            AssemblyOutput::Object(object) => {
                // 2048 is just a best guess as to the size of the file
                let mut file_buffer = Vec::with_capacity(2048);

                // Actually write to the buffer
                object.write(&mut file_buffer);

                let output_path = config
                    .output_path
                    .unwrap_or_else(|| config.input_path.with_extension(".ko"));

                let mut output_file = try_create_file(&output_path);

                if let Err(e) = output_file.write_all(&file_buffer) {
                    eprintln!(
                        "Error writing to `{}`: {}",
                        output_path.to_string_lossy(),
                        e
                    );

                    process::exit(4);
                }
            }
            AssemblyOutput::Source(source) => {
                let output_path = config
                    .output_path
                    .unwrap_or_else(|| config.input_path.with_extension(".ksm"));

                let mut output_file = try_create_file(&output_path);

                if let Err(e) = output_file.write_all(source.as_bytes()) {
                    eprintln!(
                        "Error writing to `{}`: {}",
                        output_path.to_string_lossy(),
                        e
                    );

                    process::exit(3);
                }
            }
        }
    } else {
        process::exit(1);
    }
}

fn try_create_file(path: &Path) -> File {
    match File::create(&path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error creating `{}`: {}", path.to_string_lossy(), e);

            process::exit(2);
        }
    }
}
