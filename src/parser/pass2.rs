use std::{error::Error, slice::Iter, iter::Peekable};

use crate::{Token, TokenType, TokenData, LabelManager, LabelType, LabelInfo, LabelValue, Label, OperandType, Instruction};

use kerbalobjects::{KOFile, RelInstruction, KOSValue, Symbol, SymbolInfo, SymbolType};
use kerbalobjects::{RelSection, SubrtSection};

/// This function performas the second pass of a two-pass assembler.
/// It takes instructions and outputs a KerbalObject file as the result
pub fn pass2(tokens: &Vec<Token>, label_manager: &mut LabelManager) -> Result<KOFile, Box<dyn Error>> {
    let mut kofile = KOFile::new();
    let mut token_iter = tokens.iter().peekable();
    let mut location_counter = 1;
    let mut current_label = Label::new("", LabelType::UNDEF, LabelInfo::LOCAL, LabelValue::NONE);
    let mut first_function = true;
    let mut instruction_list = Vec::new();
    let mut function_start_location = 1;

    // We want to loop through all of the tokens, so don't stop until we are out
    while token_iter.peek().is_some() {
        // Check if it is a newline, which we ignore
        if token_iter.peek().unwrap().tt() == TokenType::NEWLINE {
            // Just consume it
            token_iter.next();
        }
        // It has to be a mnemonic then
        else {
            let new_function;
            let temp_tuple;

            // We need to check if this is the start of a new function or not
            temp_tuple = is_in_new_func(location_counter, label_manager);
            new_function = temp_tuple.0;

            println!("\t\tLocation counter: {}", location_counter);
            println!("\t\tNew function?: {}", new_function);
            
            // If this is a new function
            if new_function {
                // Set the function start location
                function_start_location = location_counter;

                // If this is the first function, then we don't need to add any section to anything
                if first_function {
                    first_function = false;
                }
                // If it isn't, then we need to make a new KOFile section that contains all of the collected instructions
                else {

                    add_instructions_to_file(&current_label, instruction_list, &mut kofile);

                }

                // Set the new function label
                current_label = label_manager.get(&temp_tuple.1)?.to_owned();

                // Now we need to create a new instruction list
                instruction_list = init_section_instructions(&current_label, &mut kofile);

            }

            // Now that we have that figured out

            // KSM functions need a labelreset after the first real instruction
            // So if this is the second real instruction, we need to insert a labelreset
            // We don't need this if it is the main function though.
            if location_counter == function_start_location + 1 && current_label.id() != "_start" {
                let location_counter_string = format!("@{:0>4}", location_counter);
                instruction_list.push(create_instr(0xf0, vec![ KOSValue::STRING(location_counter_string) ], &mut kofile));
            } 
            
            let mnemonic = match token_iter.next().unwrap().data() { TokenData::STRING(s) => s, _ => unreachable!() };

            let opcode = Instruction::opcode_from_mnemonic(mnemonic);
            let possible_types_list = Instruction::operands_from_opcode(opcode);

            let mut operand_tokens = Vec::new();
            let mut operand_kos_values = Vec::new();
            let instr;

            // Now we need to consume all of the operands
            // This will keep going until we hit a newline
            while token_iter.peek().is_some() && token_iter.peek().unwrap().tt() != TokenType::NEWLINE {

                // Collect the token
                let token = token_iter.next().unwrap();

                // Push it
                operand_tokens.push(token.clone());

                // Is the next token a comma?
                if token_iter.peek().is_some() && token_iter.peek().unwrap().tt() == TokenType::COMMA {
                    // Consume it
                    token_iter.next();
                }                

            }

            // Now that we have all of the operands, we need to convert them to KOSValues
            for (index, token) in operand_tokens.iter().enumerate() {
                let possible_types = possible_types_list.get(index).unwrap();
                let kos_value = token_to_kosvalue(token, possible_types, label_manager)?;

                operand_kos_values.push(kos_value);
            }

            // Finally we need to create an instruction from this, and push it to the current list
            instr = create_instr(opcode, operand_kos_values, &mut kofile);

            instruction_list.push(instr);

            // As long as this wasn't a label reset instruction
            if opcode != 0xf0 {
                // Then increment the location counter by 1
                location_counter += 1;
            }

        }
    }

    // After this, we will have the instructions from the very last function in the instruction list
    add_instructions_to_file(&current_label, instruction_list, &mut kofile);

    Ok(kofile)
}

fn add_instructions_to_file(current_label: &Label, instruction_list: Vec<RelInstruction>, kofile: &mut KOFile) {
    println!("\t\tCreating new function: {}", current_label.id());

    // Now we need to check if it is the main section or a subroutine
    if current_label.id() == "_start" {
        // If it is, then we need to make a RelSection with the contents
        let mut main_text = RelSection::new(".text");

        // Now we add each instruction to the main section
        for instr in instruction_list {
            main_text.add(instr);
        }

        // And set the KOFile's main text as this section
        kofile.set_main_text(main_text);
    }
    // If it is a regular subroutine
    else {
        // Make a new subroutine section with the name of the current function
        let mut subrt_section = SubrtSection::new(current_label.id());

        // Now we add each instruction to the section
        for instr in instruction_list {
            subrt_section.add(instr);
        }

        // Add it
        kofile.add_subrt_section(subrt_section);
    }
}

/// This function chooses the correct operand type to create from the given token
fn best_operand_type(token: &Token, possible_types: &Vec<OperandType>) -> Result<OperandType, Box<dyn Error>> {
    Ok(match token.tt() {
        TokenType::IDENTIFIER => {
            // This could still be either a label or a boolean value
            let value = match token.data() { TokenData::STRING(s) => s, _ => unreachable!() };

            // Check if it is true or false
            if value == "true" || value == "false" {
                // We still need to check though if the instruction wants a bool, or boolvalue
                if possible_types.contains(&OperandType::BOOL) {
                    OperandType::BOOL
                } else {
                    OperandType::BOOLEANVALUE
                }
            } else {
                if possible_types.contains(&OperandType::STRING) {
                    OperandType::STRING
                } else {
                    OperandType::STRINGVALUE
                }
            }
        },
        TokenType::DOUBLE => {
            OperandType::SCALARDOUBLE
        },
        TokenType::ATSYMBOL => {
            OperandType::ARGMARKER
        },
        TokenType::HASH => {
            OperandType::NULL
        },
        TokenType::INT => {
            // This will always return the smallest possible to fit the number
            let value = match token.data() { TokenData::INT(i) => *i, _ => unreachable!() };

            if possible_types.contains(&OperandType::BYTE) && (value < 127 && value > -128) {
                OperandType::BYTE
            }
            else if possible_types.contains(&OperandType::INT16) && (value < 32767 && value > -32768) {
                OperandType::INT16
            }
            else if possible_types.contains(&OperandType::INT32) {
                OperandType::INT32
            }
            else if possible_types.contains(&OperandType::SCALARINT) {
                OperandType::SCALARINT
            }
            else {
                // If we have reached this point, it actually just means that the value is greater than the max value of an int32
                // This is an error
                return Err(format!("Value {} is greater than the maximum value storable. Line {}", value, token.line()).into());
            }
        },
        TokenType::STRING => {
            if possible_types.contains(&OperandType::STRING) {
                OperandType::STRING
            } else {
                OperandType::STRINGVALUE
            }
        },
        _ => {
            panic!("Invalid token {} found during Pass 2!", token.as_str())
        }
    })
}

/// This function creates a KOSValue based on accepted operand types and the current token
fn token_to_kosvalue(token: &Token, possible_types: &Vec<OperandType>, label_manager: &mut LabelManager) -> Result<KOSValue, Box<dyn Error>> {
    let best_type = best_operand_type(token, possible_types)?;

    // This makes all of the later lines much easier.
    let str_value = match token.data() { TokenData::STRING(s) => Some(s), _ => None };
    let double_value = match token.data() { TokenData::DOUBLE(d) => Some(*d), _ => None };
    let int_value =  match token.data() { TokenData::INT(i) => Some(*i), _ => None };

    match best_type {
        OperandType::BOOL => {
            Ok(KOSValue::BOOL(str_value.unwrap() == "true"))
        },
        OperandType::BOOLEANVALUE => {
            Ok(KOSValue::BOOLEANVALUE(str_value.unwrap() == "true"))
        },
        OperandType::STRING | OperandType::STRINGVALUE => {
            let value_to_save;
            // For this we need to test if the token is an identifier that is a label
            if token.tt() == TokenType::IDENTIFIER && label_manager.ifdef(str_value.unwrap()) {
                // If it is, we need to get the label's value
                let label_value = label_manager.get(str_value.unwrap())?.label_value();
                let label_str = match label_value { LabelValue::STRING(s) => s, _=> unreachable!() };

                // Then we return a KOSValue containing that
                value_to_save = label_str.to_owned();
            } else {
                // If not, then just ust the value itself
                value_to_save = str_value.unwrap().to_owned()
            }

            // Finally, return the KOSValue
            if best_type == OperandType::STRING {
                Ok(KOSValue::STRING(value_to_save))
            } else {
                Ok(KOSValue::STRINGVALUE(value_to_save))
            }
        },
        OperandType::SCALARDOUBLE => {
            Ok(KOSValue::SCALARDOUBLE(double_value.unwrap()))
        },
        OperandType::ARGMARKER => {
            Ok(KOSValue::ARGMARKER)
        },
        OperandType::NULL => {
            Ok(KOSValue::NULL)
        },
        OperandType::BYTE => {
            Ok(KOSValue::BYTE(int_value.unwrap() as i8))
        },
        OperandType::INT16 => {
            Ok(KOSValue::INT16(int_value.unwrap() as i16))
        },
        OperandType::INT32 => {
            Ok(KOSValue::INT32(int_value.unwrap()))
        },
        OperandType::SCALARINT => {
            Ok(KOSValue::SCALARINT(int_value.unwrap()))
        },
    }
}

/// Creates a new vector of RelInstructions to start the new section given by func_label
fn init_section_instructions(func_label: &Label, kofile: &mut KOFile) -> Vec<RelInstruction> {
    // Create the vector
    let mut instr = Vec::new();

    let label_value = match func_label.label_value() { LabelValue::STRING(s) => s.to_owned(), _ => unreachable!() };

    // Push the lbrt for this section
    instr.push(create_instr(0xf0, vec![KOSValue::STRING(label_value)], kofile));

    instr
}

/// Creates a RelInstruction created from the given opcode and operands
/// Also serves to save used symbols to the symbol table of the KOFile
fn create_instr(opcode: u8, operands: Vec<KOSValue>, kofile: &mut KOFile) -> RelInstruction {
    let mut operand_symbols = Vec::new();

    // Loop through each operand
    for operand in operands.iter() {
        // 2 is the symbol data section
        let symbol = Symbol::new("", operand.clone(), operand.size(), SymbolInfo::LOCAL, SymbolType::NOTYPE, 2);

        let symbol_index = kofile.add_symbol(symbol) as u32;

        operand_symbols.push(symbol_index);
    }

    RelInstruction::new(opcode, operand_symbols)
}

/// Tests if we have entered a new function
/// Returns a tuple containing true or false, and the function's label's id
/// If the answer is no, then the label is just an empty string
/// It also replaces the function's label's value with the function's name
fn is_in_new_func(location_counter: u32, label_manager: &mut LabelManager) -> (bool, String) {
    // Create the lc string we would expect
    let lc_string = format!("@{:0>4}", location_counter);

    println!("\t\tLC string: {}", lc_string);

    // If it is a label's value, then 
    let label_option = label_manager.contains_value(LabelValue::STRING(lc_string.to_owned()));

    match label_option {
        Some(label) => {
            // Check if it is a function at all
            if label.label_type() == LabelType::FUNC {
                // If this is _start, then we should leave it how it is
                if label.id() != "_start" {
                    // If not though, change the label's value to the id
                    let new_label = Label::new(label.id(), label.label_type(), label.label_info(), LabelValue::STRING(label.id().to_owned()));

                    // Redefine the label
                    label_manager.def(label.id(), new_label);
                }
                // If it is start, or we are done changing the label then just return true
                (true, label.id().to_owned())
            }
            // If not, that is a no
            else {
                (false, String::new())
            }
        },
        None => {
            (false, String::new())
        }
    }
}