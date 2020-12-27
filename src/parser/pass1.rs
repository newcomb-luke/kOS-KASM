use std::{error::Error, iter::Peekable, slice::Iter};

use crate::{
    ExpressionEvaluator, ExpressionParser, Instruction, Label, LabelInfo, LabelManager, LabelType,
    LabelValue, OperandType, Token, TokenData, TokenType, Value, ValueType,
};

/// This function performas the first pass of a two-pass assembler.
/// It stores and evaluates labels, and also evaluates any expressions
pub fn pass1(
    tokens: &Vec<Token>,
    label_manager: &mut LabelManager,
) -> Result<Vec<Token>, Box<dyn Error>> {
    let mut token_iter = tokens.iter().peekable();
    let mut new_tokens = Vec::new();
    let mut last_newline = true;
    let mut create_label_next = false;
    let mut label_id = String::new();
    let mut last_label_id = String::new();
    let mut location_counter = 1;
    let mut lc_string = String::new();
    let mut instructions_valid = false;

    // We want to loop through all of the tokens, so don't stop until we are out
    while token_iter.peek().is_some() {
        let token = token_iter.next().unwrap();

        // If this is a newline
        if token.tt() == TokenType::NEWLINE {
            // I figure we can do a little optimization and not repeat large portions of newlines
            if !last_newline {
                // If the last token was not a newline, push this one as usual and set the flag
                new_tokens.push(token.clone());
                last_newline = true;
            }
        } else {
            // If not, set the flag to false
            last_newline = false;

            // Check if the token is an identifier
            if token.tt() == TokenType::IDENTIFIER {
                // Extract the identifier
                let id = match token.data() {
                    TokenData::STRING(s) => s,
                    _ => unreachable!(),
                };
                let possible_operands;
                let new_operands;
                let instr_opcode = Instruction::opcode_from_mnemonic(id);

                // Check if it is a mnemonic, because at this point it MUST be
                if instr_opcode == 0 {
                    return Err(format!(
                        "Identifier {} is not a valid instruction. Line {}",
                        id,
                        token.line()
                    )
                    .into());
                }

                // Get the possible operand combinations from the instruction
                possible_operands = Instruction::operands_from_opcode(instr_opcode);

                // Now we call a function to read all operands, verify them, and possibly evaluate them
                new_operands = read_and_verify_operands(
                    id,
                    token.line(),
                    &mut token_iter,
                    &possible_operands,
                    label_manager,
                    &last_label_id,
                )?;

                // Now we unconditionally push both the instruction and operands, comma separated
                new_tokens.push(token.clone());

                for (idx, op_token) in new_operands.iter().enumerate() {
                    // Push the operand
                    new_tokens.push(op_token.clone());
                    // If this isn't the last, then we need to put a comma
                    if idx < new_operands.len() - 1 {
                        new_tokens.push(Token::new(TokenType::COMMA, TokenData::NONE));
                    }
                }

                // Finally, we need to check if this was a label reset instruction, which is not a real instruction and does not count for the LC
                if instr_opcode == 0xf0 {
                    // If it is a label reset, then set the LC string for next iteration
                    lc_string = match new_operands.get(0).unwrap().data() {
                        TokenData::STRING(s) => s.to_owned(),
                        _ => unreachable!(),
                    };
                }
                // If it is a regular instruction
                else {
                    // We need to check if we are supposed to create a Label that points to this instruction
                    if create_label_next {
                        let new_label;
                        let label_value;
                        // Then create a Label

                        // Check the LC string
                        if !lc_string.is_empty() {
                            // If it isn't empty, there is a special label value to use
                            label_value = lc_string.to_owned();
                        }
                        // If it is empty
                        else {
                            // This Label's value will be @00LC
                            label_value = format!("@{:0>4}", location_counter);
                        }

                        // We actually only want to create a Label in the case that no Label exists with this id
                        // We should first check if one exists
                        if label_manager.ifdef(&label_id) {
                            // If it does exist, then we need to check which type.
                            let original_label = label_manager.get(&label_id)?;
                            let original_type = original_label.label_type();

                            // If the Label type is anything other than undefined, then that is an error because the same Label has been declared somewhere else
                            // Also it is a duplicate if the info is external
                            if (original_label.label_type() != LabelType::UNDEF
                                && original_label.label_type() != LabelType::UNDEFFUNC)
                                || original_label.label_info() == LabelInfo::EXTERN
                            {
                                return Err(format!("Duplicate Label {} already exists. Found declared again. Line {}", label_id, token.line()).into());
                            }

                            // If not, just update the label to be a regular label or function
                            if original_type == LabelType::UNDEF {
                                new_label = Label::new(
                                    &label_id,
                                    LabelType::DEF,
                                    original_label.label_info(),
                                    LabelValue::STRING(label_value),
                                );
                            } else {
                                // Now instructions are allowed
                                instructions_valid = true;
                                new_label = Label::new(
                                    &label_id,
                                    LabelType::FUNC,
                                    original_label.label_info(),
                                    LabelValue::STRING(label_value),
                                );
                            }
                        }
                        // If it isn't already defined
                        else {
                            new_label = Label::new(
                                &label_id,
                                LabelType::DEF,
                                LabelInfo::LOCAL,
                                LabelValue::STRING(label_value),
                            );
                        }

                        // Finally, add the Label to the manager
                        label_manager.def(&label_id, new_label);

                        // Clear the flag
                        create_label_next = false;
                    }

                    // Clear the lc_string
                    lc_string = String::new();

                    // Add 1 to the LC
                    location_counter += 1;
                }

                // Due to the structure required of KSM files, we cannot have any instructions that are outside of either a function or label
                if !instructions_valid {
                    return Err(format!(
                        "Instruction {} is not inside of a function. Line {}",
                        id,
                        token.line()
                    )
                    .into());
                }
            }
            // If it is an inner label
            else if token.tt() == TokenType::INNERLABEL {
                // Get the label string
                let inner_label = match token.data() {
                    TokenData::STRING(s) => s,
                    _ => unreachable!(),
                };

                // Check if there was a regular label before this
                if last_label_id.is_empty() {
                    // That is an error
                    return Err(format!(
                        "Lone local label found {}. Line {}",
                        inner_label,
                        token.line()
                    )
                    .into());
                }

                // Set Label id to the correct value
                label_id = format!("{}.{}", last_label_id, inner_label);

                // Set the flag to create this Label next time
                create_label_next = true;
            }
            // If this is a regular label
            else if token.tt() == TokenType::LABEL {
                // Get the label string
                let label_str = match token.data() {
                    TokenData::STRING(s) => s,
                    _ => unreachable!(),
                };

                // Overwrite both the last Label id, and this Label id
                last_label_id = label_str.to_owned();
                label_id = label_str.to_owned();

                // Set the flag to create this Label next time
                create_label_next = true;
            }
        }
    }

    // The final job of pass 1 is to check for any undefined local or global Labels
    for label in label_manager.as_vec().iter() {
        // Check if it is not external and undefined
        if label.label_info() != LabelInfo::EXTERN
            && (label.label_type() == LabelType::UNDEF
                || label.label_type() == LabelType::UNDEFFUNC)
        {
            // If it is, then it is an undefined Label and we need to throw an error
            return Err(format!("Undefined label {}.", label.id()).into());
        }
    }

    Ok(new_tokens)
}

fn read_and_verify_operands(
    instr_id: &str,
    instr_line: usize,
    token_iter: &mut Peekable<Iter<Token>>,
    possible_operands: &Vec<Vec<OperandType>>,
    label_manager: &mut LabelManager,
    last_label: &String,
) -> Result<Vec<Token>, Box<dyn Error>> {
    let mut read_operands = Vec::new();
    let mut evaluated_operands = Vec::new();
    let mut reached_end = false;

    // If we start with a newline as the next token, then there are no operands
    if token_iter.peek().is_some() && token_iter.peek().unwrap().tt() != TokenType::NEWLINE {
        // Just for fun, check if the first token is a comma, because that would be a problem
        if token_iter.peek().unwrap().tt() == TokenType::COMMA {
            return Err(format!(
                "Error parsing instruction, expected operand and found , instead. Line {}",
                instr_line
            )
            .into());
        }

        // We will need to read all supplied operands from the token iterator
        while token_iter.peek().is_some() && !reached_end {
            let mut operand_tokens = Vec::new();

            // Just for fun, check if the next token is a comma, because that would be a problem
            if token_iter.peek().unwrap().tt() == TokenType::COMMA {
                return Err(format!(
                    "Error parsing instruction, expected operand and found , instead. Line {}",
                    instr_line
                )
                .into());
            }

            // Loop through until we reach the end, or either a comma or newline
            while token_iter.peek().is_some()
                && token_iter.peek().unwrap().tt() != TokenType::NEWLINE
                && token_iter.peek().unwrap().tt() != TokenType::COMMA
            {
                operand_tokens.push(token_iter.next().unwrap().clone());
            }

            // If this ended, it is one of three reasons, two of which mean that we are done
            if token_iter.peek().is_none() || token_iter.peek().unwrap().tt() == TokenType::NEWLINE
            {
                reached_end = true;
            } else {
                // If we are here, we ended because there was a comma
                // So consume it
                token_iter.next();
            }

            // Either way, push the operand
            read_operands.push(operand_tokens);
        }
    }

    // Now that we have the operands, we need to check them

    // The most obvious thing to check would be the amount
    if read_operands.len() != possible_operands.len() {
        return Err(format!(
            "Instruction {} requires {} operands, {} supplied. Line {}",
            instr_id,
            possible_operands.len(),
            read_operands.len(),
            instr_line
        )
        .into());
    }

    // We do not support adding constants to labels
    // So the choice for operands is either an identifier, an expression, an @, a #, or a string.
    // NO combinations of them

    for (index, operand) in read_operands.iter().enumerate() {
        let mut operand_accepted = false;
        let first_token = operand.get(0).unwrap();

        // If it is a string
        if first_token.tt() == TokenType::STRING {
            // The first thing we should do is check if that is an acceptable type
            for operand_possibility in possible_operands.get(index).unwrap().iter() {
                if *operand_possibility == OperandType::STRINGVALUE
                    || *operand_possibility == OperandType::STRING
                {
                    operand_accepted = true;
                }
            }

            // All we can do is push it
            evaluated_operands.push(first_token.clone());

            // We also need to check that there is nothing else here
            if operand.len() > 1 {
                return Err(format!("Found other tokens in operand. If operand is a string, it must only contain the string. Line {}", instr_line).into());
            }
        }
        // If it is a "directive"
        else if first_token.tt() == TokenType::DIRECTIVE {
            // A "directive" at this stage would actually be something like this:
            // jmp .loopend
            // That is just a reference to a local label!

            // In order to fix this, actually replace it with an identifier
            let inner_label_id = match first_token.data() {
                TokenData::STRING(s) => s,
                _ => unreachable!(),
            };
            let label_id;
            let new_identifier;

            // Because only certain instruction accept labels, we need to test this
            for operand_possibility in possible_operands.get(index).unwrap().iter() {
                if *operand_possibility == OperandType::STRINGVALUE
                    || *operand_possibility == OperandType::STRING
                {
                    operand_accepted = true;
                }
            }

            // Now using the last label, we need to create the full Label id for this
            label_id = format!("{}.{}", last_label, inner_label_id);

            // We also need to make an entry in the Label table for this
            if !label_manager.ifdef(&label_id) {
                label_manager.def(
                    &label_id,
                    Label::new(
                        &label_id,
                        LabelType::UNDEF,
                        LabelInfo::LOCAL,
                        LabelValue::NONE,
                    ),
                );
            }

            // Now we just create the identifier token
            new_identifier = Token::new(TokenType::IDENTIFIER, TokenData::STRING(label_id));

            // All we can do is push it now
            evaluated_operands.push(new_identifier);

            // We also need to check that there is nothing else here
            if operand.len() > 1 {
                return Err(format!("Found other tokens in operand. If operand is a label, it must only contain the label. Line {}", instr_line).into());
            }
        }
        // If it is an identifier
        else if first_token.tt() == TokenType::IDENTIFIER {
            // Retrieve the id
            let identifier_id = match first_token.data() {
                TokenData::STRING(s) => s,
                _ => unreachable!(),
            };

            // We need to check if it is a boolean value, which is an identifier
            if identifier_id != "true" && identifier_id != "false" {
                // This could either mean data or a label, but either way is acceptable.
                operand_accepted = true;

                // We do however need to make an entry in the Label table if it doesn't already exist
                if !label_manager.ifdef(identifier_id) {
                    label_manager.def(
                        identifier_id,
                        Label::new(
                            identifier_id,
                            LabelType::UNDEF,
                            LabelInfo::LOCAL,
                            LabelValue::NONE,
                        ),
                    );
                }

                // All we can do is push it
                evaluated_operands.push(first_token.clone());

                // We also need to check that there is nothing else here
                if operand.len() > 1 {
                    return Err(format!("Found other tokens in operand. If operand is a Label, it must only contain the Label. Line {}", instr_line).into());
                }
            }
            // If it was true or false
            else {
                // This might actually be an expression...
                // First we need to make the operand into an expression, and evaluate it
                let expression_result = operand_to_expression_result(operand, index, instr_id)?;

                // We need to verify if the expression results in the type that we want
                operand_accepted =
                    is_expression_acceptable(&expression_result, index, &possible_operands);

                // Then we need to push a new token corresponding to the result
                evaluated_operands.push(result_to_token(&expression_result));
            }
        }
        // If it is a @ (argument marker)
        else if first_token.tt() == TokenType::ATSYMBOL {
            // The first thing we should do is check if that is an acceptable type
            for operand_possibility in possible_operands.get(index).unwrap().iter() {
                if *operand_possibility == OperandType::ARGMARKER {
                    operand_accepted = true;
                }
            }

            // All we can do is push it
            evaluated_operands.push(first_token.clone());

            // We also need to check that there is nothing else here
            if operand.len() > 1 {
                return Err(format!("Found other tokens in operand. If operand is an argument marker, it must only contain the argument marker. Line {}", instr_line).into());
            }
        }
        // If it is a # (null value)
        else if first_token.tt() == TokenType::HASH {
            // The first thing we should do is check if that is an acceptable type
            for operand_possibility in possible_operands.get(index).unwrap().iter() {
                if *operand_possibility == OperandType::NULL {
                    operand_accepted = true;
                }
            }

            // All we can do is push it
            evaluated_operands.push(first_token.clone());

            // We also need to check that there is nothing else here
            if operand.len() > 1 {
                return Err(format!("Found other tokens in operand. If operand is a null value, it must only contain the null value. Line {}", instr_line).into());
            }
        }
        // If this is supposed to be an expression of some kind
        else {
            // First we need to make the operand into an expression, and evaluate it
            let expression_result = operand_to_expression_result(operand, index, instr_id)?;

            // We need to verify if the expression results in the type that we want
            operand_accepted =
                is_expression_acceptable(&expression_result, index, &possible_operands);

            // Then we need to push a new token corresponding to the result
            evaluated_operands.push(result_to_token(&expression_result));
        }

        // If the operand was not accepted, then we need to print an error
        if !operand_accepted {
            let mut accepted_list_str = String::new();

            // Add each possible operand type to the string
            for (op_index, operand_possibility) in
                possible_operands.get(index).unwrap().iter().enumerate()
            {
                accepted_list_str.push_str(&format!(" {:?}", operand_possibility));

                // If this isn't the last one
                if op_index < possible_operands.get(index).unwrap().len() - 1 {
                    // Add a nice comma
                    accepted_list_str.push(',');
                }
            }

            return Err(format!("Operand {} for instruction {} is of the wrong type. Accepted types are:{}. Line {}", index, instr_id, accepted_list_str, instr_line).into());
        }

        // If it was accepted we just move on to the next iteration
    }

    Ok(evaluated_operands)
}

/// Turns an operand as a list of tokens into a parsed and evaluated expression result value
fn operand_to_expression_result(
    operand: &Vec<Token>,
    index: usize,
    mnemonic: &str,
) -> Result<Value, Box<dyn Error>> {
    let mut expression_iter = operand.iter().peekable();
    let expression;
    let result;

    // Parse the expression
    expression = match ExpressionParser::parse_expression(&mut expression_iter) {
        Ok(expression) => expression.unwrap(),
        Err(e) => {
            return Err(format!(
            "Expected expression as argument {} for instruction {}. Expression parsing failed: {}",
            index, mnemonic, e
        )
            .into())
        }
    };

    // Then evaluate it
    result = match ExpressionEvaluator::evaluate(&expression) {
        Ok(result) => result,
        Err(e) => return Err(format!("Expected expression as argument {} for instruction {}. Expression evaluation failed: {}", index, mnemonic, e).into()),
    };

    Ok(result)
}

fn is_expression_acceptable(
    operand_result: &Value,
    index: usize,
    possible_operands: &Vec<Vec<OperandType>>,
) -> bool {
    let mut operand_accepted = false;
    // Now that we have the answer, we need to check its type to see if it is acceptable
    for operand_possibility in possible_operands.get(index).unwrap().iter() {
        // If the result is a boolean
        if operand_result.valtype() == ValueType::BOOL {
            // If the required type is a boolean, we are good
            if *operand_possibility == OperandType::BOOL
                || *operand_possibility == OperandType::BOOLEANVALUE
            {
                operand_accepted = true;
            }
        }
        // If the result is an int
        else if operand_result.valtype() == ValueType::INT {
            operand_accepted |= match *operand_possibility {
                OperandType::NULL => false,
                OperandType::BOOL => false,
                OperandType::BYTE => true,
                OperandType::INT16 => true,
                OperandType::INT32 => true,
                OperandType::ARGMARKER => false,
                OperandType::SCALARINT => true,
                OperandType::SCALARDOUBLE => false,
                OperandType::BOOLEANVALUE => false,
                OperandType::STRING => false,
                OperandType::STRINGVALUE => false,
            }
        }
        // If the result is a double
        else if operand_result.valtype() == ValueType::DOUBLE {
            if *operand_possibility == OperandType::SCALARDOUBLE {
                operand_accepted = true;
            }
        }
    }

    operand_accepted
}

fn result_to_token(result: &Value) -> Token {
    match result.valtype() {
        ValueType::BOOL => Token::new(
            TokenType::IDENTIFIER,
            TokenData::STRING(String::from(if result.to_bool().ok().unwrap() {
                "true"
            } else {
                "false"
            })),
        ),
        ValueType::DOUBLE => Token::new(
            TokenType::DOUBLE,
            TokenData::DOUBLE(result.to_double().ok().unwrap()),
        ),
        ValueType::INT => Token::new(
            TokenType::INT,
            TokenData::INT(result.to_int().ok().unwrap()),
        ),
    }
}
