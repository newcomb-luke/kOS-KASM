use std::{
    fmt::{Display, Formatter},
    iter::Peekable,
    slice::Iter,
};

use kerbalobjects::KOSValue;

use crate::{
    ExpressionEvaluator, ExpressionParser, InstructionParseError, InstructionParseResult, Token,
    TokenData, TokenType, Value, ValueType,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperandType {
    NULL,
    BOOL,
    BYTE,
    INT16,
    INT32,
    // FLOAT,
    DOUBLE,
    STRING,
    ARGMARKER,
    SCALARINT,
    SCALARDOUBLE,
    BOOLEANVALUE,
    STRINGVALUE,
}

#[derive(Debug, Clone)]
pub enum Operand {
    VALUE(KOSValue),
    LABELREF(String),
}

pub struct Instruction {
    opcode: u8,
    operands: Vec<Operand>,
}

impl Instruction {
    /// Creates a new Instruction from the given information
    pub fn new(opcode: u8, operands: Vec<Operand>) -> Instruction {
        Instruction { opcode, operands }
    }

    pub fn opcode(&self) -> u8 {
        self.opcode
    }

    pub fn operands(&self) -> &Vec<Operand> {
        &self.operands
    }

    /// Parses a new instruction from the given tokens
    pub fn parse(
        parent_label_id: &str,
        token_iter: &mut Peekable<Iter<Token>>,
    ) -> InstructionParseResult<Instruction> {
        let mut opcode;
        let operands;
        let mnemonic = match token_iter.next().unwrap().data() {
            TokenData::STRING(s) => s,
            _ => unreachable!(),
        };
        let possible_types;
        let token_operands;

        opcode = Instruction::opcode_from_mnemonic(mnemonic);
        possible_types = Instruction::operand_types_from_opcode(opcode);

        // If the returned opcode is 0, that means that the identifier is actually not an instruction mnemonic
        if opcode == 0 {
            return Err(InstructionParseError::InvalidInstructionError(
                mnemonic.to_owned(),
            ));
        }

        // Collect the operand tokens
        token_operands = Instruction::gather_operands(token_iter)?;

        // Process the operand tokens into operands
        operands = Instruction::process_operands(parent_label_id, possible_types, token_operands)?;

        // If all of that went smoothly, let us check if we are the fake pushv instruction, and correct it
        if opcode == 0xfa {
            opcode = 0x4e;
        }

        Ok(Instruction::new(opcode, operands))
    }

    /// This function verifies, evaluates, and converts the operands given
    fn process_operands(
        parent_label_id: &str,
        possible_types: Vec<Vec<OperandType>>,
        token_operands: Vec<Vec<Token>>,
    ) -> InstructionParseResult<Vec<Operand>> {
        let mut new_operands = Vec::new();

        // The most obvious thing to check is the amount of operands
        if possible_types.len() != token_operands.len() {
            return Err(InstructionParseError::NumOperandsMismatchError(
                possible_types.len(),
                token_operands.len(),
            ));
        }

        // We do not support adding constants to labels
        // So the choice for operands is either an identifier, an expression, an @, a #, or a string.
        // NO combinations of them

        for (op_index, operand) in token_operands.iter().enumerate() {
            let first_token = operand.get(0).unwrap();
            let operand_accepted;

            // This is printed in the event of an error
            let mut accepted_list_str = String::new();

            // Add each possible operand type to the string
            for (idx, operand_possibility) in
                possible_types.get(op_index).unwrap().iter().enumerate()
            {
                accepted_list_str.push_str(&format!(" {:?}", operand_possibility));

                // If this isn't the last one
                if idx < possible_types.get(op_index).unwrap().len() - 1 {
                    // Add a nice comma
                    accepted_list_str.push(',');
                }
            }

            match first_token.tt() {
                TokenType::STRING => {
                    let kosvalue;
                    let value = match first_token.data() {
                        TokenData::STRING(s) => s,
                        _ => unreachable!(),
                    };

                    if possible_types
                        .get(op_index)
                        .unwrap()
                        .contains(&OperandType::STRING)
                    {
                        kosvalue = KOSValue::STRING(value.to_owned());
                        operand_accepted = true;
                    } else if possible_types
                        .get(op_index)
                        .unwrap()
                        .contains(&OperandType::STRINGVALUE)
                    {
                        kosvalue = KOSValue::STRINGVALUE(value.to_owned());
                        operand_accepted = true;
                    } else {
                        kosvalue = KOSValue::NULL;
                        operand_accepted = false;
                    }

                    // I feel like there should be a better way than having to check this...
                    if operand_accepted {
                        // Add the operand
                        new_operands.push(Operand::VALUE(kosvalue));

                        // Now we need to make sure there is nothing else
                        Instruction::assert_single_token(operand, "string")?;
                    }
                }
                TokenType::DIRECTIVE => {
                    // A "directive" at this stage would actually be something like this:
                    // jmp .loopend
                    // That is just a reference to a local label!

                    let inner_label_id = match first_token.data() {
                        TokenData::STRING(s) => s,
                        _ => unreachable!(),
                    };
                    let full_label_id;

                    // Because only certain instruction accept labels, we need to test this
                    operand_accepted = possible_types
                        .get(op_index)
                        .unwrap()
                        .contains(&OperandType::INT32);

                    if operand_accepted {
                        // Now using the last label, we need to create the full Label id for this
                        full_label_id = format!("{}.{}", parent_label_id, inner_label_id);

                        // We also need to make an entry in the label manager for this, but that will come later

                        // Now we just create the operand
                        new_operands.push(Operand::LABELREF(full_label_id));

                        Instruction::assert_single_token(operand, "label")?;
                    }
                }
                TokenType::IDENTIFIER => {
                    // Retrieve the value itself
                    let label_id = match first_token.data() {
                        TokenData::STRING(s) => s,
                        _ => unreachable!(),
                    };

                    // An identifier at this stage would be a label
                    operand_accepted = possible_types
                        .get(op_index)
                        .unwrap()
                        .contains(&OperandType::INT32)
                        || possible_types
                            .get(op_index)
                            .unwrap()
                            .contains(&OperandType::STRING);

                    if operand_accepted {
                        // Basically just add it back as it came
                        new_operands.push(Operand::LABELREF(label_id.to_owned()));

                        Instruction::assert_single_token(operand, "label")?;
                    }
                }
                // If it is a @ (argument marker)
                TokenType::ATSYMBOL => {
                    // We first need to check if that is acceptable
                    operand_accepted = possible_types
                        .get(op_index)
                        .unwrap()
                        .contains(&OperandType::ARGMARKER);

                    if operand_accepted {
                        new_operands.push(Operand::VALUE(KOSValue::ARGMARKER));

                        Instruction::assert_single_token(operand, "argument marker")?;
                    }
                }
                // If it is a # (null value)
                TokenType::HASH => {
                    // We first need to check if that is acceptable
                    operand_accepted = possible_types
                        .get(op_index)
                        .unwrap()
                        .contains(&OperandType::NULL);

                    if operand_accepted {
                        new_operands.push(Operand::VALUE(KOSValue::NULL));

                        Instruction::assert_single_token(operand, "null value")?;
                    }
                }
                // Anything else, and this is an expression that needs to be evaluated
                _ => {
                    let mut expression_iter = operand.iter().peekable();

                    // First we need to make the operand into an expression
                    let expression = match ExpressionParser::parse_expression(&mut expression_iter)
                    {
                        Ok(exp) => exp,
                        Err(e) => {
                            return Err(InstructionParseError::ExpressionParseFailedError(
                                op_index, e,
                            ));
                        }
                    };

                    // Then we need to evaluate it
                    let expression_result = match ExpressionEvaluator::evaluate(&expression) {
                        Ok(result) => result,
                        Err(e) => {
                            return Err(InstructionParseError::ExpressionEvalFailedError(
                                op_index, e,
                            ));
                        }
                    };

                    // Turn this result into a KOSValue
                    let operand_kosvalue = match Instruction::get_correct_operand(
                        expression_result,
                        possible_types.get(op_index).unwrap(),
                    ) {
                        Ok(op) => {
                            operand_accepted = true;
                            op
                        }
                        Err(e) => match e {
                            InstructionParseError::InternalOperandNotAcceptedError => {
                                operand_accepted = false;
                                KOSValue::NULL
                            }
                            InstructionParseError::InternalOperandTooLargeError => {
                                return Err(InstructionParseError::IntOperandTooLargeError(
                                    accepted_list_str,
                                ));
                            }
                            _ => unreachable!(),
                        },
                    };

                    // Add it to the list
                    new_operands.push(Operand::VALUE(operand_kosvalue));
                }
            }

            // If the operand was not accepted we need to return an error
            if !operand_accepted {
                let mut accepted_list_str = String::new();

                // Add each possible operand type to the string
                for (op_index, operand_possibility) in
                    possible_types.get(op_index).unwrap().iter().enumerate()
                {
                    accepted_list_str.push_str(&format!(" {:?}", operand_possibility));

                    // If this isn't the last one
                    if op_index < possible_types.get(op_index).unwrap().len() - 1 {
                        // Add a nice comma
                        accepted_list_str.push(',');
                    }
                }

                return Err(InstructionParseError::InvalidOperandTypeError(
                    op_index + 1,
                    accepted_list_str,
                ));
            }
        }

        Ok(new_operands)
    }

    // This function takes a value and matches it with possible operand types to produce an operand with the correct kOSValue
    fn get_correct_operand(
        value: Value,
        possible_types: &Vec<OperandType>,
    ) -> InstructionParseResult<KOSValue> {
        match value.valtype() {
            ValueType::INT => {
                let int_value = value.to_int();

                // If the possible types don't exist at all in the list, then we need to return an error
                if !possible_types.contains(&OperandType::BYTE)
                    && !possible_types.contains(&OperandType::INT16)
                    && !possible_types.contains(&OperandType::INT32)
                    && !possible_types.contains(&OperandType::SCALARINT)
                {
                    Err(InstructionParseError::InternalOperandNotAcceptedError)
                } else {
                    match Instruction::get_smallest_int_type(value, possible_types) {
                        Ok(op_type) => Ok(match op_type {
                            OperandType::BYTE => KOSValue::BYTE(int_value as i8),
                            OperandType::INT16 => KOSValue::INT16(int_value as i16),
                            OperandType::INT32 => KOSValue::INT32(int_value),
                            OperandType::SCALARINT => KOSValue::SCALARINT(int_value),
                            _ => unreachable!(),
                        }),
                        Err(_) => Err(InstructionParseError::InternalOperandTooLargeError),
                    }
                }
            }
            ValueType::BOOL => {
                let bool_value = value.to_bool();
                // This will default to the non-value version
                if possible_types.contains(&OperandType::BOOL) {
                    Ok(KOSValue::BOOL(bool_value))
                } else if possible_types.contains(&OperandType::BOOLEANVALUE) {
                    Ok(KOSValue::BOOLEANVALUE(bool_value))
                } else {
                    Err(InstructionParseError::InternalOperandNotAcceptedError)
                }
            }
            ValueType::DOUBLE => {
                let double_value = value.to_double();
                // This will also default to the non-value version
                if possible_types.contains(&OperandType::DOUBLE) {
                    Ok(KOSValue::DOUBLE(double_value))
                } else if possible_types.contains(&OperandType::SCALARDOUBLE) {
                    Ok(KOSValue::SCALARDOUBLE(double_value))
                } else {
                    Err(InstructionParseError::InternalOperandNotAcceptedError)
                }
            }
        }
    }

    /// This function returns the smallest OperandType that is an integer that the instruction supports
    fn get_smallest_int_type(
        value: Value,
        possible_types: &Vec<OperandType>,
    ) -> InstructionParseResult<OperandType> {
        // This finds the smallest possible type that can store the integer provided
        let int_value = value.to_int();
        let byte_max = 127;
        let byte_min = -128;
        let int16_max = 32767;
        let int16_min = -32768;

        if (int_value > byte_min && int_value < byte_max)
            && possible_types.contains(&OperandType::BYTE)
        {
            Ok(OperandType::BYTE)
        } else if (int_value > int16_min && int_value < int16_max)
            && possible_types.contains(&OperandType::INT16)
        {
            Ok(OperandType::INT16)
        } else if possible_types.contains(&OperandType::INT32) {
            Ok(OperandType::INT32)
        } else if possible_types.contains(&OperandType::SCALARINT) {
            Ok(OperandType::SCALARINT)
        } else {
            Err(InstructionParseError::InternalOperandTooLargeError)
        }
    }

    /// This function checks if there are more than one tokens in the vector, and if so, it returns an error, if not, then it returns nothing
    fn assert_single_token(operand: &Vec<Token>, operand_name: &str) -> InstructionParseResult<()> {
        if operand.len() > 1 {
            Err(InstructionParseError::ExtraTokensInOperandError(
                operand_name.to_owned(),
            ))
        } else {
            Ok(())
        }
    }

    /// This function consumes the rest of the tokens on an instruction line, in order to gather them as operands
    fn gather_operands(
        token_iter: &mut Peekable<Iter<Token>>,
    ) -> InstructionParseResult<Vec<Vec<Token>>> {
        let mut operands = Vec::new();
        let mut ended_with_comma = false;

        // Read all tokens until we reach an EOF or newline
        while token_iter.peek().is_some() && token_iter.peek().unwrap().tt() != TokenType::NEWLINE {
            let mut operand = Vec::new();

            // Now loop until we reach a newline, EOF, or a comma
            while token_iter.peek().is_some()
                && token_iter.peek().unwrap().tt() != TokenType::NEWLINE
                && token_iter.peek().unwrap().tt() != TokenType::COMMA
            {
                // Add it to the operands list
                operand.push(token_iter.next().unwrap().clone());
            }

            if operand.is_empty() {
                if ended_with_comma {
                    // If we did end last with a comma, and the operand is empty, then that isn't right
                    return Err(InstructionParseError::ExpectedOperandError);
                }
            } else {
                operands.push(operand);
            }

            // If we ended with a comma, then consume it
            if token_iter.peek().is_some() && token_iter.peek().unwrap().tt() == TokenType::COMMA {
                token_iter.next();
                ended_with_comma = true;
            }
            // If we didn't
            else {
                ended_with_comma = false;
            }
        }

        Ok(operands)
    }

    /// Returns true of the given string is a mnemonic of an instruction, false otherwise.
    pub fn is_instruction(mnemonic: &str) -> bool {
        let opcode = Instruction::opcode_from_mnemonic(mnemonic);
        // If the opcode is 0 that means it is bogus, or not an instruction
        opcode != 0
    }

    pub fn opcode_from_mnemonic(mnemonic: &str) -> u8 {
        match mnemonic {
            "eof" => 0x31,
            "eop" => 0x32,
            "nop" => 0x33,
            "sto" => 0x34,
            "uns" => 0x35,
            "gmb" => 0x36,
            "smb" => 0x37,
            "gidx" => 0x38,
            "sidx" => 0x39,
            "bfa" => 0x3a,
            "jmp" => 0x3b,
            "add" => 0x3c,
            "sub" => 0x3d,
            "mul" => 0x3e,
            "div" => 0x3f,
            "pow" => 0x40,
            "cgt" => 0x41,
            "clt" => 0x42,
            "cge" => 0x43,
            "cle" => 0x44,
            "ceq" => 0x45,
            "cne" => 0x46,
            "neg" => 0x47,
            "bool" => 0x48,
            "not" => 0x49,
            "and" => 0x4a,
            "or" => 0x4b,
            "call" => 0x4c,
            "ret" => 0x4d,
            "push" => 0x4e,
            "pop" => 0x4f,
            "dup" => 0x50,
            "swap" => 0x51,
            "eval" => 0x52,
            "addt" => 0x53,
            "rmvt" => 0x54,
            "wait" => 0x55,
            "gmet" => 0x57,
            "stol" => 0x58,
            "stog" => 0x59,
            "bscp" => 0x5a,
            "escp" => 0x5b,
            "stoe" => 0x5c,
            "phdl" => 0x5d,
            "btr" => 0x5e,
            "exst" => 0x5f,
            "argb" => 0x60,
            "targ" => 0x61,
            "tcan" => 0x62,

            "prl" => 0xce,
            "pdrl" => 0xcd,
            "lbrt" => 0xf0,

            // This had to be added to be able to do anything in kOS that you can do with normal kerbalscript
            // It is a "fake" instruction that will push the "value" type of any compatible type
            // Opcode fa for fake :)
            "pushv" => 0xfa,

            _ => 0x00,
        }
    }

    // Returns a vector of vectors representing the different operand types that each instruction can take
    pub fn operand_types_from_opcode(opcode: u8) -> Vec<Vec<OperandType>> {
        match opcode {
            0x31 => vec![],
            0x32 => vec![],
            0x33 => vec![],
            0x34 => vec![vec![OperandType::STRING]],
            0x35 => vec![],
            0x36 => vec![vec![OperandType::STRING]],
            0x37 => vec![vec![OperandType::STRING]],
            0x38 => vec![],
            0x39 => vec![],
            0x3a => vec![vec![OperandType::STRING, OperandType::INT32]],
            0x3b => vec![vec![OperandType::STRING, OperandType::INT32]],
            0x3c => vec![],
            0x3d => vec![],
            0x3e => vec![],
            0x3f => vec![],
            0x40 => vec![],
            0x41 => vec![],
            0x42 => vec![],
            0x43 => vec![],
            0x44 => vec![],
            0x45 => vec![],
            0x46 => vec![],
            0x47 => vec![],
            0x48 => vec![],
            0x49 => vec![],
            0x4a => vec![],
            0x4b => vec![],
            0x4c => vec![
                vec![OperandType::STRING],
                vec![
                    OperandType::STRING,
                    OperandType::INT16,
                    OperandType::INT32,
                    OperandType::NULL,
                ],
            ],
            0x4d => vec![vec![OperandType::INT16]],
            0x4e => vec![vec![
                OperandType::NULL,
                OperandType::BOOL,
                OperandType::BYTE,
                OperandType::INT16,
                OperandType::INT32,
                OperandType::STRING,
                OperandType::ARGMARKER,
                OperandType::DOUBLE,
            ]],
            0x4f => vec![],
            0x50 => vec![],
            0x51 => vec![],
            0x52 => vec![],
            0x53 => vec![vec![OperandType::BOOL], vec![OperandType::INT32]],
            0x54 => vec![],
            0x55 => vec![],
            0x56 => vec![],
            0x57 => vec![vec![OperandType::STRING]],
            0x58 => vec![vec![OperandType::STRING]],
            0x59 => vec![vec![OperandType::STRING]],
            0x5a => vec![vec![OperandType::INT16], vec![OperandType::INT16]],
            0x5b => vec![vec![OperandType::INT16]],
            0x5c => vec![vec![OperandType::STRING]],
            0x5d => vec![vec![
                OperandType::BYTE,
                OperandType::INT16,
                OperandType::INT32,
            ]],
            0x5e => vec![vec![OperandType::STRING, OperandType::INT32]],
            0x5f => vec![],
            0x60 => vec![],
            0x61 => vec![],
            0x62 => vec![],

            0xce => vec![vec![OperandType::STRING]],
            0xcd => vec![vec![OperandType::STRING], vec![OperandType::BOOL]],
            0xf0 => vec![vec![OperandType::STRING]],

            // Fake instruction
            0xfa => vec![vec![
                OperandType::STRINGVALUE,
                OperandType::BOOLEANVALUE,
                OperandType::SCALARINT,
                OperandType::SCALARDOUBLE,
            ]],
            _ => vec![],
        }
    }
}

impl Display for Operand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut op_str = String::new();

        for (idx, op) in self.operands.iter().enumerate() {
            op_str.push_str(&format!("{}", op));

            if idx < self.operands.len() - 1 {
                op_str.push_str(", ");
            }
        }

        write!(f, "{:x} {}", self.opcode, op_str)
    }
}
