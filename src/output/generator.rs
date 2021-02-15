use crate::{Function, Instruction, LabelManager, LabelType, LabelValue, Operand};
use kerbalobjects::{KOFile, KOSValue, RelInstruction, RelSection, Symbol, SymbolInfo, SymbolType};

use super::{GeneratorError, GeneratorResult};

const FIRST_FUNC_SECTION: usize = 4;

#[derive(Debug)]
pub struct Generator {}

impl Generator {
    pub fn generate(
        functions: Vec<Function>,
        label_manager: &mut LabelManager,
    ) -> GeneratorResult<KOFile> {
        let mut kofile = KOFile::new();

        // First we will populate the symbols for the functions
        for (idx, func) in functions.iter().enumerate() {

            let func_name = func.name();            

            let symbol_name = if func.name() == "_start" {
                ".text"
            } else if func.name() == "_init" {
                ".init"
            } else {
                &func_name
            };

            let func_symbol = Symbol::new(
                symbol_name,
                KOSValue::NULL,
                func.size(),
                SymbolInfo::GLOBAL,
                SymbolType::FUNC,
                idx + FIRST_FUNC_SECTION,
            );

            kofile.add_symbol(func_symbol);
        }

        let mut location_counter = 0;
        for func in functions {
            let rel_section = Generator::generate_function(func, &mut kofile, label_manager, &mut location_counter)?;

            kofile.add_code_section(rel_section);
        }

        Ok(kofile)
    }

    fn generate_function(func: Function, kofile: &mut KOFile, label_manager: &LabelManager, location_counter: &mut u32) -> GeneratorResult<RelSection> {
        let func_name = func.name();            

        let section_name = if func.name() == "_start" {
            ".text"
        } else if func.name() == "_init" {
            ".init"
        } else {
            &func_name
        };
        
        let mut code_section = RelSection::new(section_name);

        for instr in func.instructions() {
            let rel_instruction = Generator::instr_to_rel(instr, kofile, label_manager, *location_counter)?;

            if rel_instruction.get_opcode() != 0xf0 {
                *location_counter += 1;
            }

            code_section.add(rel_instruction);
        }

        Ok(code_section)
    }

    fn instr_to_rel(instr: &Instruction, kofile: &mut KOFile, label_manager: &LabelManager, location_counter: u32) -> GeneratorResult<RelInstruction> {
        let mut rel_operands = Vec::new();

        for op in instr.operands() {
            let rel_op = match op {
                Operand::VALUE(v) => {
                    let value_sym = Symbol::new(
                        "",
                        v.clone(),
                        v.size(),
                        SymbolInfo::LOCAL,
                        SymbolType::NOTYPE,
                        2,
                    );

                    kofile.add_symbol(value_sym) as u32
                }
                Operand::LABELREF(f) => {

                    let label = label_manager.get(f).unwrap();

                    let sym_index;

                    if label.label_type() == LabelType::FUNC {

                        sym_index = match kofile.get_symtab().get_index_by_name(f) {
                            Ok(idx) => idx,
                            Err(_) => return Err(GeneratorError::UnresolvedFuncRefError(f.to_owned())),
                        };

                    }
                    else {

                        let label_location = match label.label_value() { LabelValue::LOC(l) => *l, _ => unreachable!() };
                        let rel_jump_loc = label_location - location_counter;

                        let loc_sym = Symbol::new("", KOSValue::INT32(rel_jump_loc as i32), 4, SymbolInfo::LOCAL, SymbolType::NOTYPE, 2);

                        sym_index = kofile.add_symbol(loc_sym);
                    }

                    sym_index as u32
                }
            };

            rel_operands.push(rel_op);
        }

        Ok(RelInstruction::new(instr.opcode(), rel_operands))
    }
}
