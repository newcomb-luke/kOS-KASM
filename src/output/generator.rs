use crate::{Function, Instruction, LabelManager, LabelType, LabelValue, Operand};

use super::errors::{GeneratorError, GeneratorResult};

use kerbalobjects::{
    kofile::{
        sections::{DataSection, RelSection, StringTable, SymbolTable},
        symbols::KOSymbol,
        symbols::SymBind,
        symbols::SymType,
        Instr, KOFile,
    },
    KOSValue, Opcode,
};

#[derive(Debug)]
pub struct Generator {}

impl Generator {
    pub fn generate(
        functions: Vec<Function>,
        label_manager: &mut LabelManager,
    ) -> GeneratorResult<KOFile> {
        let mut kofile = KOFile::new();

        let mut symtab = kofile.new_symtab(".symtab");
        let mut datatab = kofile.new_datasection(".data");
        let mut symstrtab = kofile.new_strtab(".symstrtab");
        let start_index = kofile.section_count();
        let mut rel_sections = Vec::new();

        // First we will populate the symbols for the functions
        for (idx, func) in functions.iter().enumerate() {
            let func_name = func.name();

            let symbol_name = if func_name == "_start" {
                ".text"
            } else if func_name == "_init" {
                ".init"
            } else {
                &func_name
            };
            let func_name_idx = symstrtab.add(symbol_name);

            let sh_idx = start_index + idx;
            let mut func_symbol = KOSymbol::new(
                0,
                func.size(),
                SymBind::Global,
                SymType::Func,
                sh_idx as u16,
            );
            func_symbol.set_name_idx(func_name_idx);

            symtab.add(func_symbol);
        }

        let mut location_counter = 0;
        for func in functions {
            let rel_section = Generator::generate_function(
                func,
                &mut kofile,
                &mut datatab,
                &mut symtab,
                &mut symstrtab,
                label_manager,
                &mut location_counter,
            )?;

            rel_sections.push(rel_section);
        }

        kofile.add_str_tab(symstrtab);
        kofile.add_data_section(datatab);
        kofile.add_sym_tab(symtab);

        for rel_section in rel_sections {
            kofile.add_rel_section(rel_section);
        }

        Ok(kofile)
    }

    fn generate_function(
        func: Function,
        kofile: &mut KOFile,
        data_section: &mut DataSection,
        symtab: &mut SymbolTable,
        symstrtab: &mut StringTable,
        label_manager: &LabelManager,
        location_counter: &mut u32,
    ) -> GeneratorResult<RelSection> {
        let func_name = func.name();

        let section_name = if func.name() == "_start" {
            ".text"
        } else if func.name() == "_init" {
            ".init"
        } else {
            &func_name
        };

        let mut code_section = kofile.new_relsection(section_name);

        for instr in func.instructions() {
            let ko_instr = Generator::instr_to_koinstr(
                instr,
                data_section,
                symtab,
                symstrtab,
                label_manager,
                *location_counter,
            )?;

            if ko_instr.opcode() == Opcode::Lbrt {
                *location_counter += 1;
            }

            code_section.add(ko_instr);
        }

        Ok(code_section)
    }

    fn instr_to_koinstr(
        instr: &Instruction,
        data_section: &mut DataSection,
        symtab: &mut SymbolTable,
        symstrtab: &mut StringTable,
        label_manager: &LabelManager,
        location_counter: u32,
    ) -> GeneratorResult<Instr> {
        let mut instr_operands = Vec::new();

        for op in instr.operands() {
            let instr_op = match op {
                Operand::VALUE(v) => {
                    let val_idx = data_section.add_checked(v.clone());

                    let val_sym = KOSymbol::new(
                        val_idx,
                        v.size_bytes() as u16,
                        SymBind::Local,
                        SymType::NoType,
                        data_section.section_index() as u16,
                    );

                    symtab.add_checked(val_sym)
                }
                Operand::LABELREF(f) => {
                    let label = label_manager.get(f).unwrap();

                    let sym_index;

                    if label.label_type() == LabelType::FUNC {
                        let func_name_idx = symstrtab.find(f);

                        sym_index = match func_name_idx {
                            Some(name_idx) => symtab
                                .find(symtab.find_has_name(name_idx).unwrap())
                                .unwrap(),
                            None => {
                                return Err(GeneratorError::UnresolvedFuncRefError(f.to_owned()))
                            }
                        };
                    } else {
                        let label_location = match label.label_value() {
                            LabelValue::LOC(l) => *l,
                            LabelValue::NONE => {
                                return Err(GeneratorError::EmptyLabelError(label.id().to_owned()));
                            }
                        };
                        let rel_jump_loc = label_location - location_counter;

                        let value = KOSValue::Int32(rel_jump_loc as i32);
                        let value_size = value.size_bytes();
                        let value_idx = data_section.add_checked(value);

                        let loc_sym = KOSymbol::new(
                            value_idx,
                            value_size as u16,
                            SymBind::Local,
                            SymType::NoType,
                            data_section.section_index() as u16,
                        );

                        sym_index = symtab.add_checked(loc_sym);
                    }

                    sym_index
                }
            };

            instr_operands.push(instr_op);
        }

        let opcode = Opcode::from(instr.opcode());

        Ok(match instr_operands.len() {
            0 => Instr::ZeroOp(opcode),
            1 => Instr::OneOp(opcode, instr_operands[0]),
            _ => Instr::TwoOp(opcode, instr_operands[0], instr_operands[1]),
        })
    }
}
