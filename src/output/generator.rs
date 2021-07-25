use crate::{Function, Instruction, LabelManager, LabelType, LabelValue, Operand};

use super::errors::{GeneratorError, GeneratorResult};

use kerbalobjects::{
    kofile::{
        sections::{DataSection, FuncSection, ReldSection, SectionIndex, StringTable, SymbolTable},
        symbols::SymBind,
        symbols::SymType,
        symbols::{KOSymbol, ReldEntry},
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
        let mut reld_section = kofile.new_reldsection(".reld");
        let mut symstrtab = kofile.new_strtab(".symstrtab");
        let start_index = kofile.section_count();
        let mut func_sections = Vec::new();

        // First we will populate the symbols for the functions
        for (idx, func) in functions.iter().enumerate() {
            let func_name_idx = symstrtab.add(&func.name());

            let sh_idx = start_index + idx;
            let mut func_symbol = KOSymbol::new(
                0,
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
            let func_section = Generator::generate_function(
                func,
                &mut kofile,
                &mut datatab,
                &mut reld_section,
                &mut symtab,
                &mut symstrtab,
                label_manager,
                &mut location_counter,
            )?;

            func_sections.push(func_section);
        }

        kofile.add_str_tab(symstrtab);
        kofile.add_data_section(datatab);
        kofile.add_sym_tab(symtab);

        for func_section in func_sections {
            kofile.add_func_section(func_section);
        }

        Ok(kofile)
    }

    fn generate_function(
        func: Function,
        kofile: &mut KOFile,
        data_section: &mut DataSection,
        reld_section: &mut ReldSection,
        symtab: &mut SymbolTable,
        symstrtab: &mut StringTable,
        label_manager: &LabelManager,
        location_counter: &mut u32,
    ) -> GeneratorResult<FuncSection> {
        let mut code_section = kofile.new_funcsection(&func.name());
        let section_index = code_section.section_index();

        for (instr_index, instr) in func.instructions().iter().enumerate() {
            let ko_instr = Generator::instr_to_koinstr(
                instr,
                instr_index,
                section_index,
                data_section,
                reld_section,
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
        instr_index: usize,
        section_index: usize,
        data_section: &mut DataSection,
        reld_section: &mut ReldSection,
        symtab: &mut SymbolTable,
        symstrtab: &mut StringTable,
        label_manager: &LabelManager,
        location_counter: u32,
    ) -> GeneratorResult<Instr> {
        let mut instr_operands = Vec::new();

        for (op_index, op) in instr.operands().iter().enumerate() {
            let instr_op = match op {
                Operand::VALUE(v) => data_section.add_checked(v.clone()),
                Operand::LABELREF(f) => {
                    let label = label_manager.get(f).unwrap();

                    let sym_index = if label.label_type() == LabelType::FUNC {
                        let func_name_idx = symstrtab
                            .find(f)
                            .ok_or(GeneratorError::UnresolvedFuncRefError(f.to_owned()))?;

                        symtab.position_by_name(func_name_idx).unwrap()
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

                        let name_index = symstrtab.add_checked(f);

                        let sym = KOSymbol::new(
                            name_index,
                            value_idx,
                            value_size as u16,
                            SymBind::Local,
                            SymType::NoType,
                            data_section.section_index() as u16,
                        );

                        symtab.add(sym)
                    };

                    let reld_entry =
                        ReldEntry::new(section_index, instr_index, op_index, sym_index);

                    reld_section.add(reld_entry);

                    0
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
