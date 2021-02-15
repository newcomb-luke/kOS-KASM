use std::{iter::Peekable, slice::Iter};

use super::errors::ParseError;
use crate::{
    Instruction, Label, LabelInfo, LabelManager, LabelType, LabelValue, ParseResult, Token,
    TokenData, TokenType,
};

pub struct Function {
    name: String,
    instructions: Vec<Instruction>,
    size: u16,
}

impl Function {
    pub fn new(name: &str, instructions: Vec<Instruction>, size: u16) -> Function {
        Function {
            name: name.to_owned(),
            instructions,
            size,
        }
    }

    pub fn parse(
        token_iter: &mut Peekable<Iter<Token>>,
        location_counter: &mut u32,
        label_manager: &mut LabelManager,
    ) -> ParseResult<Function> {
        // The next token has to be the function's label
        let func_name = match token_iter.next().unwrap().data() {
            TokenData::STRING(s) => s,
            _ => unreachable!(),
        };
        let mut instructions = Vec::new();

        let func_info = label_manager.get(func_name).unwrap().label_info();

        label_manager.def(
            func_name,
            Label::new(
                func_name,
                LabelType::FUNC,
                func_info,
                LabelValue::LOC(*location_counter),
            ),
        );

        println!("Parsing function {}", func_name);

        let mut parent_label_id = func_name.to_owned();
        let mut size = 0;

        // Loop through each token
        while token_iter.peek().is_some() && token_iter.peek().unwrap().tt() != TokenType::FUNCTION
        {
            let token = (*token_iter.peek().unwrap()).clone();

            match token.tt() {
                TokenType::NEWLINE => {
                    token_iter.next();
                }
                TokenType::LABEL => {
                    let label_id = match token.data() {
                        TokenData::STRING(s) => s,
                        _ => unreachable!(),
                    };

                    if !label_manager.ifdef(label_id) {
                        // Make a new local label
                        let new_label = Label::new(
                            label_id,
                            LabelType::DEF,
                            LabelInfo::LOCAL,
                            LabelValue::LOC(*location_counter),
                        );

                        // Store it
                        label_manager.def(label_id, new_label);
                    } else {
                        let old_label = label_manager.get(label_id).unwrap();

                        // If it was already defined, but also with a value, then it is a duplicate
                        if old_label.label_type() != LabelType::UNDEF {
                            return Err(ParseError::DuplicateLabelError(
                                label_id.to_owned(),
                                token.line(),
                            ));
                        }

                        let new_label = Label::new(
                            label_id,
                            LabelType::DEF,
                            old_label.label_info(),
                            LabelValue::LOC(*location_counter),
                        );

                        parent_label_id = label_id.to_owned();

                        label_manager.def(label_id, new_label);
                    }

                    token_iter.next();
                }
                TokenType::INNERLABEL => {
                    let label_suffix = match token.data() {
                        TokenData::STRING(s) => s,
                        _ => unreachable!(),
                    };
                    let label_id = format!("{}.{}", parent_label_id, label_suffix);

                    if !label_manager.ifdef(&label_id) {
                        // Make a new local label
                        let new_label = Label::new(
                            &label_id,
                            LabelType::DEF,
                            LabelInfo::LOCAL,
                            LabelValue::LOC(*location_counter),
                        );

                        // Store it
                        label_manager.def(&label_id, new_label);
                    } else {
                        let old_label = label_manager.get(&label_id).unwrap();

                        // If it was already defined, but also with a value, then it is a duplicate
                        if old_label.label_type() != LabelType::UNDEF {
                            return Err(ParseError::DuplicateLabelError(label_id, token.line()));
                        }

                        let new_label = Label::new(
                            &label_id,
                            LabelType::DEF,
                            old_label.label_info(),
                            LabelValue::LOC(*location_counter),
                        );

                        label_manager.def(&label_id, new_label);
                    }

                    token_iter.next();
                }
                TokenType::IDENTIFIER => {
                    let instr = match Instruction::parse(&parent_label_id, token_iter) {
                        Ok(instr) => instr,
                        Err(e) => {
                            return Err(ParseError::InstructionParseFailed(e, token.line()));
                        }
                    };

                    // Opcode 0xf0 is LabelReset, which does not count as an instruction
                    if instr.opcode() != 0xf0 {
                        *location_counter += 1;
                    }

                    size += 1;

                    instructions.push(instr);
                }
                _ => unreachable!(),
            }
        }

        for instr in &instructions {
            println!("{}", instr);
        }

        Ok(Function::new(func_name, instructions, size))
    }

    pub fn instructions(&self) -> &Vec<Instruction> {
        &self.instructions
    }

    pub fn name(&self) -> String {
        self.name.to_owned()
    }

    pub fn size(&self) -> u16 {
        self.size
    }
}
