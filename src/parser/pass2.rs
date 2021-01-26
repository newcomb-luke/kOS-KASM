use std::error::Error;

use crate::{
    preprocessor::PreprocessError, Instruction, Label, LabelInfo, LabelManager, LabelType,
    LabelValue, OperandType, Token, TokenData, TokenType,
};

use kerbalobjects::RelSection;
use kerbalobjects::{KOFile, KOSValue, RelInstruction, Symbol, SymbolInfo, SymbolType};

/// This function performas the second pass of a two-pass assembler.
/// It takes instructions and outputs a KerbalObject file as the result
pub fn pass2(
    tokens: &Vec<Token>,
    label_manager: &mut LabelManager,
) -> Result<KOFile, Box<dyn Error>> {
    let mut kofile = KOFile::new();
    let mut token_iter = tokens.iter().peekable();
    let mut location_counter = 1;
    let mut current_func_label = None;
    let mut instruction_list = Vec::new();
    let mut current_section_index = 4;

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

            // If this is a new function
            if new_function {
                // If this is the first function, then we don't need to add any section to anything
                // If it isn't, then we need to make a new KOFile section that contains all of the collected instructions
                if current_func_label.is_some() {
                    add_instructions_to_file(&current_func_label, instruction_list, &mut kofile);
                }

                println!("Setting function label");
                // Set the new function label
                current_func_label = Some(match label_manager.get(&temp_tuple.1) {
                    Some(label) => label.to_owned(),
                    None => unreachable!(),
                });

                // Now we need to create a new instruction list
                instruction_list = Vec::new();

                let func_label = current_func_label.clone().unwrap();
                // All functions must be defined in the object file by adding a symbol
                let symbol_info = match func_label.label_info() {
                    LabelInfo::LOCAL => SymbolInfo::LOCAL,
                    LabelInfo::GLOBAL => SymbolInfo::GLOBAL,
                    LabelInfo::EXTERN => SymbolInfo::EXTERN,
                };

                // Create the symbol
                let func_symbol = Symbol::new(
                    func_label.id(),
                    KOSValue::NULL,
                    0,
                    symbol_info,
                    SymbolType::FUNC,
                    current_section_index,
                );

                current_section_index += 1;

                // Add it to the symbol table
                kofile.add_symbol(func_symbol);
            }

            // Now that we have that figured out

            // Check again if this is a new function...
            if new_function {}

            let mnemonic = match token_iter.next().unwrap().data() {
                TokenData::STRING(s) => s,
                _ => unreachable!(),
            };

            let mut opcode = Instruction::opcode_from_mnemonic(mnemonic);
            let possible_types_list = Instruction::operands_from_opcode(opcode);

            let mut operand_tokens = Vec::new();
            let mut operand_symbols = Vec::new();
            let instr;

            // Now we need to consume all of the operands
            // This will keep going until we hit a newline
            while token_iter.peek().is_some()
                && token_iter.peek().unwrap().tt() != TokenType::NEWLINE
            {
                // Collect the token
                let token = token_iter.next().unwrap();

                // Push it
                operand_tokens.push(token.clone());

                // Is the next token a comma?
                if token_iter.peek().is_some()
                    && token_iter.peek().unwrap().tt() == TokenType::COMMA
                {
                    // Consume it
                    token_iter.next();
                }
            }

            // Now that we have all of the operands, we need to convert them to KOSValues
            for (index, token) in operand_tokens.iter().enumerate() {
                let possible_types = possible_types_list.get(index).unwrap();
                let (is_symbol, kos_value) =
                    token_to_kosvalue(token, location_counter, possible_types, label_manager)?;

                // Stores the index of the symbol that this operand references
                let symbol_index;

                // If this value is a reference to a symbol
                if is_symbol {
                    match &kos_value {
                        // If it is a string, we are trying to reference a function or external symbol
                        KOSValue::STRING(s) | KOSValue::STRINGVALUE(s) => {
                            println!("Checking if it is a function");
                            // Check if it is a function
                            let label = match label_manager.get(s) {
                                Some(label) => label,
                                None => {
                                    return Err(
                                        PreprocessError::LabelDoesNotExist(s.to_owned()).into()
                                    );
                                }
                            };

                            if label.label_info() == LabelInfo::EXTERN {
                                // If it is external then we need to make a new symbol for it
                                let extern_symbol = Symbol::new(
                                    label.id(),
                                    KOSValue::NULL,
                                    0,
                                    SymbolInfo::EXTERN,
                                    SymbolType::FUNC,
                                    0,
                                );

                                // Add it
                                symbol_index = kofile.add_symbol(extern_symbol);
                            } else {
                                symbol_index = 0;
                            }
                            // If not, it is either global or local
                            // else {
                            //     match kofile.get_symtab().get_index_by_name(&s) {

                            //     }
                            //     // Get the index and store it
                            //     symbol_index = ;
                            // }
                        }
                        // If this is an int32, then this is also not a function
                        KOSValue::INT32(_) => {
                            let kos_value_size = kos_value.size();
                            // Generate a new symbol for this
                            let new_symbol = Symbol::new(
                                "",
                                kos_value,
                                kos_value_size,
                                SymbolInfo::LOCAL,
                                SymbolType::NOTYPE,
                                2,
                            );

                            // Store the symbol in the symbol table and get the index
                            symbol_index = kofile.add_symbol(new_symbol);
                        }
                        _ => unreachable!(),
                    }
                } else {
                    let kos_value_size = kos_value.size();
                    // Generate a new symbol for this
                    let new_symbol = Symbol::new(
                        "",
                        kos_value,
                        kos_value_size,
                        SymbolInfo::LOCAL,
                        SymbolType::NOTYPE,
                        2,
                    );

                    // Store the symbol in the symbol table and get the index
                    symbol_index = kofile.add_symbol(new_symbol);
                }

                // Add the symbol index to the list
                operand_symbols.push(symbol_index as u32);
            }

            // Because of our instruction fakery in instructions.rs, we need to check if this is a "pushv" instruction
            if opcode == 0xfa {
                // All we need to do is change the opcode to 0x4e, or the regular push instruction
                opcode = 0x4e;
            }

            // Finally we need to create an instruction from this, and push it to the current list
            instr = RelInstruction::new(opcode, operand_symbols);

            instruction_list.push(instr);

            // As long as this wasn't a label reset instruction
            if opcode != 0xf0 {
                // Then increment the location counter by 1
                location_counter += 1;
            }
        }
    }

    // After this, we will have the instructions from the very last function in the instruction list
    add_instructions_to_file(&current_func_label, instruction_list, &mut kofile);

    Ok(kofile)
}

fn add_instructions_to_file(
    current_func_label: &Option<Label>,
    instruction_list: Vec<RelInstruction>,
    kofile: &mut KOFile,
) {
    let func_label = current_func_label.clone().unwrap();
    let label_id = func_label.id();
    let section_name;

    // The current label id is fine unless it is _start or _init which have special section names
    section_name = match label_id.as_str() {
        "_start" => ".text",
        "_init" => ".init",
        s => s,
    };

    // Make a new rel section with the name of the current function
    let mut code_section = RelSection::new(section_name);

    // Now we add each instruction to the section
    for instr in instruction_list {
        code_section.add(instr);
    }

    // Add it
    kofile.add_code_section(code_section);
}

/// This function chooses the correct operand type to create from the given token
fn best_operand_type(
    token: &Token,
    possible_types: &Vec<OperandType>,
    label_manager: &LabelManager,
) -> Result<(bool, OperandType), Box<dyn Error>> {
    let mut is_symbol = false;

    let op_type = match token.tt() {
        TokenType::IDENTIFIER => {
            // This could still be either a label or a boolean value
            let value = match token.data() {
                TokenData::STRING(s) => s,
                _ => unreachable!(),
            };

            // Check if it is true or false
            if value == "true" || value == "false" {
                // We still need to check though if the instruction wants a bool, or boolvalue
                if possible_types.contains(&OperandType::BOOL) {
                    OperandType::BOOL
                } else {
                    OperandType::BOOLEANVALUE
                }
            } else {
                is_symbol = true;

                println!("Checking if {} is a function in best_operand_type", value);

                // We need to check if this is a function or not
                // We will also use a string if it is undefined
                if label_manager.ifdef(value)
                    && (label_manager.get(value).unwrap().label_type() == LabelType::FUNC
                        || label_manager.get(value).unwrap().label_type() == LabelType::UNDEF)
                {
                    println!("{} is a string", value);
                    // It will only be defined if it is a function
                    // In that case, we can only use strings
                    if possible_types.contains(&OperandType::STRING) {
                        OperandType::STRING
                    } else {
                        OperandType::STRINGVALUE
                    }
                }
                // If it is not a function, then it is just another label
                else {
                    OperandType::INT32
                }
            }
        }
        TokenType::DOUBLE => {
            if possible_types.contains(&OperandType::DOUBLE) {
                OperandType::DOUBLE
            } else {
                OperandType::SCALARDOUBLE
            }
        }
        TokenType::ATSYMBOL => OperandType::ARGMARKER,
        TokenType::HASH => OperandType::NULL,
        TokenType::INT => {
            // This will always return the smallest possible to fit the number
            let value = match token.data() {
                TokenData::INT(i) => *i,
                _ => unreachable!(),
            };

            if possible_types.contains(&OperandType::BYTE) && (value < 127 && value > -128) {
                OperandType::BYTE
            } else if possible_types.contains(&OperandType::INT16)
                && (value < 32767 && value > -32768)
            {
                OperandType::INT16
            } else if possible_types.contains(&OperandType::INT32) {
                OperandType::INT32
            } else if possible_types.contains(&OperandType::SCALARINT) {
                OperandType::SCALARINT
            } else {
                // If we have reached this point, it actually just means that the value is greater than the max value of an int32
                // This is an error
                return Err(format!(
                    "Value {} is greater than the maximum value storable. Line {}",
                    value,
                    token.line()
                )
                .into());
            }
        }
        TokenType::STRING => {
            if possible_types.contains(&OperandType::STRING) {
                OperandType::STRING
            } else {
                OperandType::STRINGVALUE
            }
        }
        _ => {
            panic!("Invalid token {} found during Pass 2!", token.as_str())
        }
    };

    Ok((is_symbol, op_type))
}

/// This function creates a KOSValue based on accepted operand types and the current token
/// It stores this in a tuple with the first member being a boolean value that stores if the token was a symbol or not
fn token_to_kosvalue(
    token: &Token,
    location_counter: u32,
    possible_types: &Vec<OperandType>,
    label_manager: &mut LabelManager,
) -> Result<(bool, KOSValue), Box<dyn Error>> {
    let (is_symbol, best_type) = best_operand_type(token, possible_types, label_manager)?;

    // This makes all of the later lines much easier.
    let str_value = match token.data() {
        TokenData::STRING(s) => Some(s),
        _ => None,
    };
    let double_value = match token.data() {
        TokenData::DOUBLE(d) => Some(*d),
        _ => None,
    };
    let int_value = match token.data() {
        TokenData::INT(i) => Some(*i),
        _ => None,
    };

    let kos_value = match best_type {
        OperandType::BOOL => KOSValue::BOOL(str_value.unwrap() == "true"),
        OperandType::BOOLEANVALUE => KOSValue::BOOLEANVALUE(str_value.unwrap() == "true"),
        OperandType::STRING | OperandType::STRINGVALUE => {
            let value_to_save = str_value.unwrap().to_owned();
            // // For this we need to test if the token is an identifier that is a label
            // if token.tt() == TokenType::IDENTIFIER && label_manager.ifdef(str_value.unwrap()) {
            //     println!("Getting the label {}'s value", str_value.unwrap());
            //     // If it is, we need to get the label's value
            //     let label_value = label_manager.get(str_value.unwrap())?.label_value();
            //     let label_str;

            //     match label_value {
            //         LabelValue::STRING(s) => {
            //             label_str = s;
            //         }
            //         LabelValue::NONE => {
            //             label_str = str_value.unwrap();
            //         }
            //     };

            //     // Then we return a KOSValue containing that
            //     value_to_save = label_str.to_owned();
            // } else {
            //     // If not, then just ust the value itself
            //     value_to_save = str_value.unwrap().to_owned()
            // }

            // Finally, return the KOSValue
            if best_type == OperandType::STRING {
                KOSValue::STRING(value_to_save)
            } else {
                KOSValue::STRINGVALUE(value_to_save)
            }
        }
        OperandType::SCALARDOUBLE => KOSValue::SCALARDOUBLE(double_value.unwrap()),
        OperandType::DOUBLE => KOSValue::DOUBLE(double_value.unwrap()),
        OperandType::ARGMARKER => KOSValue::ARGMARKER,
        OperandType::NULL => KOSValue::NULL,
        OperandType::BYTE => KOSValue::BYTE(int_value.unwrap() as i8),
        OperandType::INT16 => KOSValue::INT16(int_value.unwrap() as i16),
        OperandType::INT32 => {
            // If this is a symbol, then we should check if it is coming in as an int
            if !is_symbol && token.tt() == TokenType::INT {
                KOSValue::INT32(int_value.unwrap())
            }
            // If not, we need to get the int value from this by using the location counter
            else {
                println!("Getting the label trying to get the lc");
                // This is a label, so let's get the label
                let label = label_manager.get(str_value.unwrap()).unwrap();

                let label_str = match label.label_value() {
                    LabelValue::STRING(s) => s,
                    _ => unreachable!(),
                };

                // Now we can convert the label's string into an int
                // @0042 => 42
                let label_pos: i32 = label_str[1..].parse()?;

                let rel_pos = label_pos - location_counter as i32;

                KOSValue::INT32(rel_pos)
            }
        }
        OperandType::SCALARINT => KOSValue::SCALARINT(int_value.unwrap()),
    };

    Ok((is_symbol, kos_value))
}

/// Tests if we have entered a new function
/// Returns a tuple containing true or false, and the function's label's id
/// If the answer is no, then the label is just an empty string
fn is_in_new_func(location_counter: u32, label_manager: &mut LabelManager) -> (bool, String) {
    // Create the lc string we would expect
    let lc_string = format!("@{:0>4}", location_counter);

    // If it is a label's value, then
    let label_option = label_manager.contains_value(LabelValue::STRING(lc_string.to_owned()));

    match label_option {
        Some(label) => {
            // Check if it is a function at all
            if label.label_type() == LabelType::FUNC {
                // If this is _start or _init, then we should leave it how it is
                if label.id() != "_start" && label.id() != "_init" {
                    // If not though, change the label's value to the id
                    let new_label = Label::new(
                        label.id(),
                        label.label_type(),
                        label.label_info(),
                        LabelValue::STRING(label.id().to_owned()),
                    );

                    // Redefine the label
                    label_manager.def(label.id(), new_label);
                }

                // If it is start, or we are done changing the label then just return true
                (true, label.id().to_owned())
            } else {
                (false, String::new())
            }
        }
        None => (false, String::new()),
    }
}
