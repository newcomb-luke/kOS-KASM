use std::collections::HashMap;

use kerbalobjects::kofile::{
    symbols::{KOSymbol, SymBind, SymType},
    KOFile,
};

use crate::{
    parser::{LabelManager, SymbolManager, SymbolType},
    session::Session,
};

use super::VerifiedFunction;

pub struct Generator<'a, 'b, 'c> {
    session: &'a Session,
    label_manager: &'b LabelManager,
    symbol_manager: &'c SymbolManager,
}

impl<'a, 'b, 'c> Generator<'a, 'b, 'c> {
    pub fn new(
        session: &'a Session,
        label_manager: &'b LabelManager,
        symbol_manager: &'c SymbolManager,
    ) -> Self {
        Self {
            session,
            label_manager,
            symbol_manager,
        }
    }

    pub fn generate(self, functions: Vec<VerifiedFunction>) -> Result<KOFile, ()> {
        let mut function_map: HashMap<String, u16> = HashMap::new();
        let mut functions_and_sections = Vec::with_capacity(functions.len());

        let mut ko = KOFile::new();

        let mut data_section = ko.new_datasection(".data");
        let mut sym_tab = ko.new_symtab(".symtab");
        let mut sym_str_tab = ko.new_strtab(".symstrtab");
        let mut reld_section = ko.new_reldsection(".reld");

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
                // Doesnt have a value for us to insert
                todo!();
            } else {
                // Does have a value to be inserted
                if symbol.sym_type == SymbolType::Func {
                    // If it is a function
                    let function_index = *function_map.get(name).unwrap();
                    // Here we set the size to be 0, but it will be updated later
                    let function_symbol = KOSymbol::new(
                        name_index,
                        0,
                        0,
                        symbol.binding,
                        SymType::Func,
                        function_index,
                    );
                    sym_tab.add(function_symbol);
                } else if symbol.sym_type == SymbolType::Value {
                    // If it is just a value
                    todo!();
                }
            }
        }

        todo!();
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
