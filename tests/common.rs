use kasm::{assemble_path, AssemblyOutput, Config};
use kerbalobjects::ko::symbols::{SymBind, SymType};
use kerbalobjects::ko::KOFile;
use kerbalobjects::{BufferIterator, Opcode};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub struct AssemblyTestInput {
    pub file_name_base: String,
    pub expected_symbols: Vec<(String, SymBind, SymType)>,
    pub expected_code: Vec<(String, Vec<Opcode>)>,
}

pub fn run_assembly_test(input: AssemblyTestInput) {
    let config = Config {
        emit_errors: true,
        emit_warnings: true,
        root_dir: PathBuf::from("./tests/"),
        run_preprocessor: false,
        preprocess_only: false,
        include_path: None,
        file_sym_name: None,
        comment: String::from("KASM test"),
    };

    let output = assemble_path(
        &PathBuf::from(format!("./tests/sources/{}.kasm", &input.file_name_base)),
        config,
    )
    .unwrap();

    match output {
        AssemblyOutput::Object(ko) => {
            // 2048 is just a best guess as to the size of the file
            let mut file_buffer = Vec::with_capacity(2048);

            // Actually write to the buffer
            ko.write(&mut file_buffer);

            let output_path = PathBuf::from(format!("tests/{}.ko", &input.file_name_base));

            let mut output_file = try_create_file(&output_path);

            if let Err(e) = output_file.write_all(&file_buffer) {
                panic!(
                    "Error writing to `{}`: {}",
                    output_path.to_string_lossy(),
                    e
                );
            }

            let ko = ko.get();

            for (expected_name, opcodes) in input.expected_code {
                let func = ko.func_section_by_name(expected_name).unwrap();

                for (f, &e) in func.instructions().zip(opcodes.iter()) {
                    assert_eq!(f.opcode(), e);
                }
            }

            let symtab = ko.sym_tab_by_name(".symtab").unwrap();
            let symstrtab = ko.str_tab_by_name(".symstrtab").unwrap();

            let file_name = symstrtab
                .position(format!("{}.kasm", &input.file_name_base))
                .unwrap();

            let _file = symtab.find_by_name(file_name).unwrap();

            assert_eq!(input.expected_symbols.len() + 1, symtab.symbols().len());

            for (name, bind, t) in input.expected_symbols {
                let sym_name = symstrtab.position(name).unwrap();
                let sym = symtab.find_by_name(sym_name).unwrap();

                assert_eq!(sym.sym_bind, bind);
                assert_eq!(sym.sym_type, t);
            }

            let mut input_file = OpenOptions::new().read(true).open(output_path).unwrap();

            let mut buffer = Vec::new();

            input_file.read_to_end(&mut buffer).unwrap();

            let mut buffer_iterator = BufferIterator::new(&buffer);

            let _read = KOFile::parse(&mut buffer_iterator).unwrap();
        }
        _ => panic!(),
    }
}

fn try_create_file(path: &Path) -> File {
    match File::create(path) {
        Ok(file) => file,
        Err(e) => {
            panic!("Error creating `{}`: {}", path.to_string_lossy(), e);
        }
    }
}
