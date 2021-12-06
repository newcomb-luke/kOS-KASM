use std::collections::HashMap;

use kerbalobjects::{
    kofile::{
        sections::{DataSection, FuncSection, ReldSection, SectionIndex, StringTable, SymbolTable},
        symbols::{KOSymbol, ReldEntry, SymBind, SymType},
        Instr, KOFile,
    },
    KOSValue, Opcode,
};

use crate::{
    parser::{SymbolManager, SymbolType, SymbolValue},
    session::Session,
};

use super::{VerifiedFunction, VerifiedInstruction, VerifiedOperand};

pub struct Generator<'a, 'c> {
    session: &'a Session,
    symbol_manager: &'c SymbolManager,
    global_instruction_index: usize,
}

impl<'a, 'c> Generator<'a, 'c> {
    pub fn new(session: &'a Session, symbol_manager: &'c SymbolManager) -> Self {
        Self {
            session,
            symbol_manager,
            global_instruction_index: 0,
        }
    }

    /// Generates the final object file
    pub fn generate(mut self, functions: Vec<VerifiedFunction>) -> Result<KOFile, ()> {
        let mut function_map: HashMap<String, u16> = HashMap::new();
        let mut functions_and_sections = Vec::with_capacity(functions.len());

        let mut ko = KOFile::new();

        let mut data_section = ko.new_datasection(".data");
        let mut sym_tab = ko.new_symtab(".symtab");
        let mut comment_tab = ko.new_strtab(".comment");
        let mut sym_str_tab = ko.new_strtab(".symstrtab");
        let mut reld_section = ko.new_reldsection(".reld");
        let mut function_sections = Vec::new();

        // Immediately add an initial value to the data section. This is to get around a slight
        // oversight in kerbalobject.rs where if you never reference any data, there is nothing at
        // index 0 and therefore even if the data section is never referenced, there will be a
        // linking error.
        data_section.add(KOSValue::Null);

        // Add the file's comment
        comment_tab.add(&self.session.config().comment);

        // Create all of the function sections for each function we have
        for function in functions {
            let function_section = ko.new_funcsection(&function.name);
            let function_section_index = ko.sh_index_by_name(&function.name).unwrap();

            function_map.insert(function.name.to_string(), function_section_index as u16);

            functions_and_sections.push((function_section, function));
        }
      
        // Create the file symbol
        let file_symbol_name = self.get_file_sym_name();
        let file_symbol_name_index = sym_str_tab.add(&file_symbol_name);
        let file_symbol = KOSymbol::new(
            file_symbol_name_index,
            0,
            0,
            SymBind::Global,
            SymType::File,
            0,
        );
        sym_tab.add(file_symbol);

        // Create all of the symbols
        for (name, symbol) in self.symbol_manager.symbols() {
            // Add unchecked here because we already have checked for duplicate names and there are
            // none
            let name_index = sym_str_tab.add(name);

            if symbol.binding == SymBind::Extern {
                // Doesn't have a value for us to insert

                let sym_type = match symbol.sym_type {
                    SymbolType::Func => SymType::Func,
                    SymbolType::Value => SymType::NoType,
                    _ => {
                        unreachable!();
                    }
                };

                let symbol = KOSymbol::new(name_index, 0, 0, SymBind::Extern, sym_type, 0);

                sym_tab.add(symbol);
            } else {
                // Default symbols to be local
                let bind = if symbol.binding != SymBind::Unknown {
                    symbol.binding
                } else {
                    SymBind::Local
                };

                // Does have a value to be inserted
                if symbol.sym_type == SymbolType::Func {
                    // If it is a function
                    let function_index = *function_map.get(name).unwrap();

                    // Here we set the size to be 0, but it will be updated later
                    let function_symbol =
                        KOSymbol::new(name_index, 0, 0, bind, SymType::Func, function_index);

                    sym_tab.add(function_symbol);
                } else if symbol.sym_type == SymbolType::Value {
                    // If it is just a value
                    if let SymbolValue::Value(value) = &symbol.value {
                        let size = value.size_bytes() as u16;
                        let value_index = data_section.add_checked(value.clone());

                        let symbol = KOSymbol::new(
                            name_index,
                            value_index,
                            size,
                            bind,
                            SymType::NoType,
                            data_section.section_index() as u16,
                        );

                        sym_tab.add(symbol);
                    } else {
                        self.session
                            .struct_bug(
                                "symbol had type Value, but was undefined or a function"
                                    .to_string(),
                            )
                            .emit();
                        return Err(());
                    }
                }
            }
        }

        // Now that we are done adding all of the functions and symbols, we can actually start
        // generating code
        for (func_section, function) in functions_and_sections {
            let finished = self.generate_function(
                func_section,
                function,
                &mut data_section,
                &mut reld_section,
                &sym_tab,
                &sym_str_tab,
            )?;

            function_sections.push(finished);
        }

        // Now that we have generated all of the code, we need to add the sections to the KO file
        // in the order they were created

        ko.add_data_section(data_section);
        ko.add_sym_tab(sym_tab);
        ko.add_str_tab(comment_tab);
        ko.add_str_tab(sym_str_tab);
        ko.add_reld_section(reld_section);

        for func_section in function_sections {
            ko.add_func_section(func_section);
        }

        // Finally, we are done
        if ko.update_headers().is_err() {
            self.session
                .struct_bug("Failed to update kerbal object headers".to_string())
                .emit();

            return Err(());
        }

        Ok(ko)
    }

    fn generate_function(
        &mut self,
        mut function_section: FuncSection,
        function: VerifiedFunction,
        data_section: &mut DataSection,
        reld_section: &mut ReldSection,
        sym_tab: &SymbolTable,
        sym_str_tab: &StringTable,
    ) -> Result<FuncSection, ()> {
        let function_section_index = function_section.section_index();
        let mut local_instruction_index = 0;

        for instruction in function.instructions {
            let opcode = instruction.opcode();

            let generated_instr = self.generate_instruction(
                instruction,
                function_section_index,
                local_instruction_index,
                data_section,
                reld_section,
                sym_tab,
                sym_str_tab,
            )?;

            function_section.add(generated_instr);

            if opcode != Opcode::Lbrt {
                self.global_instruction_index += 1;
            }

            local_instruction_index += 1;
        }

        Ok(function_section)
    }

    fn generate_instruction(
        &mut self,
        instruction: VerifiedInstruction,
        function_section_index: usize,
        local_instruction_index: usize,
        data_section: &mut DataSection,
        reld_section: &mut ReldSection,
        sym_tab: &SymbolTable,
        sym_str_tab: &StringTable,
    ) -> Result<Instr, ()> {
        Ok(match instruction {
            VerifiedInstruction::ZeroOp { opcode } => Instr::ZeroOp(opcode),
            VerifiedInstruction::OneOp { opcode, operand } => {
                let op = self.handle_operand(
                    operand,
                    0,
                    function_section_index,
                    local_instruction_index,
                    data_section,
                    reld_section,
                    sym_tab,
                    sym_str_tab,
                )?;
                Instr::OneOp(opcode, op)
            }
            VerifiedInstruction::TwoOp {
                opcode,
                operand1,
                operand2,
            } => {
                let op1 = self.handle_operand(
                    operand1,
                    0,
                    function_section_index,
                    local_instruction_index,
                    data_section,
                    reld_section,
                    sym_tab,
                    sym_str_tab,
                )?;
                let op2 = self.handle_operand(
                    operand2,
                    1,
                    function_section_index,
                    local_instruction_index,
                    data_section,
                    reld_section,
                    sym_tab,
                    sym_str_tab,
                )?;

                Instr::TwoOp(opcode, op1, op2)
            }
        })
    }

    fn handle_operand(
        &mut self,
        operand: VerifiedOperand,
        operand_index: usize,
        function_section_index: usize,
        local_instruction_index: usize,
        data_section: &mut DataSection,
        reld_section: &mut ReldSection,
        sym_tab: &SymbolTable,
        sym_str_tab: &StringTable,
    ) -> Result<usize, ()> {
        Ok(match operand {
            VerifiedOperand::Value(value) => data_section.add_checked(value),
            VerifiedOperand::Label(location) => {
                // Because this is an absolute location and not a relative one, we have to convert
                // it to a relative one
                let relative = location as i32 - self.global_instruction_index as i32;
                let value = KOSValue::Int32(relative);

                data_section.add_checked(value)
            }
            VerifiedOperand::Symbol(s) => {
                let name_index = sym_str_tab.find(&s).unwrap();

                let symbol_index = sym_tab.position_by_name(name_index).unwrap();

                let reld_entry = ReldEntry::new(
                    function_section_index,
                    local_instruction_index,
                    operand_index,
                    symbol_index,
                );

                reld_section.add(reld_entry);

                0
            }
        })
    }

    fn get_file_sym_name(&self) -> String {
        if let Some(name) = &self.session.config().file_sym_name {
            name.to_string()
        } else {
            // If it isn't provided, we will need to get it
            self.session.get_input_file_name()
        }
    }
}
