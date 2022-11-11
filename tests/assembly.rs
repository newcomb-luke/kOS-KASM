use kerbalobjects::ko::symbols::{SymBind, SymType};
use kerbalobjects::Opcode;

mod common;
use common::{run_assembly_test, AssemblyTestInput};

#[test]
fn externs() {
    run_assembly_test(AssemblyTestInput {
        file_name_base: String::from("externs"),
        expected_symbols: vec![
            (String::from("other"), SymBind::Extern, SymType::Func),
            (String::from("data"), SymBind::Extern, SymType::NoType),
            (
                String::from("global_value"),
                SymBind::Global,
                SymType::NoType,
            ),
            (String::from("_start"), SymBind::Global, SymType::Func),
        ],
        expected_code: vec![(String::from("_start"), vec![Opcode::Push, Opcode::Call])],
    });
}

#[test]
fn add_numbers() {
    run_assembly_test(AssemblyTestInput {
        file_name_base: String::from("add_numbers"),
        expected_symbols: vec![(String::from("_start"), SymBind::Global, SymType::Func)],
        expected_code: vec![(
            String::from("_start"),
            vec![
                Opcode::Bscp,
                Opcode::Argb,
                Opcode::Push,
                Opcode::Push,
                Opcode::Push,
                Opcode::Add,
                Opcode::Call,
                Opcode::Pop,
                Opcode::Escp,
            ],
        )],
    });
}

#[test]
fn single_instruction() {
    run_assembly_test(AssemblyTestInput {
        file_name_base: String::from("single_instruction"),
        expected_symbols: vec![(String::from("_start"), SymBind::Local, SymType::Func)],
        expected_code: vec![(String::from("_start"), vec![Opcode::Eop])],
    });
}
