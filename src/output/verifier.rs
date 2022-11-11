use std::convert::TryFrom;

use kerbalobjects::{ko::symbols::SymBind, KOSValue, Opcode};

use crate::{
    errors::Span,
    parser::{
        parse::{InstructionOperand, ParsedFunction, ParsedInstruction},
        LabelManager, SymbolManager, SymbolType, SymbolValue,
    },
    session::Session,
};

/// A verifier that verifies if all labels have been declared, all symbols have been declared, all
/// instruction operands are valid, etc.
///
/// This is the last step before code generation
///
pub struct Verifier<'a, 'b, 'c> {
    functions: Vec<ParsedFunction>,
    session: &'a Session,
    label_manager: &'b LabelManager,
    symbol_manager: &'c SymbolManager,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OperandType {
    Null,
    Bool,
    Byte,
    Int16,
    Int32,
    // Float,
    Double,
    String,
    ArgMarker,
    ScalarInt,
    ScalarDouble,
    BooleanValue,
    StringValue,

    // KASM types
    Label,
    Function,
}

impl OperandType {
    pub fn to_str(self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::Bool | Self::BooleanValue => "bool",
            Self::Byte | Self::Int16 | Self::Int32 | Self::ScalarInt => "integer",
            Self::Double | Self::ScalarDouble => "float",
            Self::String | Self::StringValue => "string",
            Self::ArgMarker => "arg marker",
            Self::Label => "label",
            Self::Function => "function label",
        }
    }
}

#[derive(Debug, Clone)]
pub enum VerifiedOperand {
    Value(KOSValue),
    Symbol(String),
    Label(usize),
}

#[derive(Debug, Clone)]
pub enum VerifiedInstruction {
    ZeroOp {
        opcode: Opcode,
    },
    OneOp {
        opcode: Opcode,
        operand: VerifiedOperand,
    },
    TwoOp {
        opcode: Opcode,
        operand1: VerifiedOperand,
        operand2: VerifiedOperand,
    },
}

impl VerifiedInstruction {
    pub fn opcode(&self) -> Opcode {
        *match self {
            Self::ZeroOp { opcode } => opcode,
            Self::OneOp { opcode, operand: _ } => opcode,
            Self::TwoOp {
                opcode,
                operand1: _,
                operand2: _,
            } => opcode,
        }
    }
}

pub struct VerifiedFunction {
    pub name: String,
    pub instructions: Vec<VerifiedInstruction>,
}

impl VerifiedFunction {
    pub fn new(name: String, instructions: Vec<VerifiedInstruction>) -> Self {
        Self { name, instructions }
    }
}

impl<'a, 'b, 'c> Verifier<'a, 'b, 'c> {
    pub fn new(
        functions: Vec<ParsedFunction>,
        session: &'a Session,
        label_manager: &'b LabelManager,
        symbol_manager: &'c SymbolManager,
    ) -> Self {
        Self {
            functions,
            session,
            label_manager,
            symbol_manager,
        }
    }

    /// Runs the verifier
    pub fn verify(self) -> Result<Vec<VerifiedFunction>, ()> {
        let mut functions = Vec::new();

        for function in self.functions.iter() {
            let verified = self.verify_function(function)?;

            functions.push(verified);
        }

        Ok(functions)
    }

    // Verifies a single function
    fn verify_function(&self, function: &ParsedFunction) -> Result<VerifiedFunction, ()> {
        let mut instructions = Vec::new();

        for instruction in function.instructions.iter() {
            let verified = self.verify_instruction(instruction)?;

            instructions.push(verified);
        }

        Ok(VerifiedFunction::new(
            function.name.to_string(),
            instructions,
        ))
    }

    // Verifies a single instruction
    fn verify_instruction(
        &self,
        instruction: &ParsedInstruction,
    ) -> Result<VerifiedInstruction, ()> {
        let mut opcode = instruction.opcode();
        let accepted_operands = self.lookup_accepted_operands(opcode)?;

        // This is a special case for if the user used a pushv instruction
        // pushv is just for the purposes of assembling, and therefore must be replaced by the
        // regular push instruction. This is done here.
        if opcode == Opcode::Pushv {
            opcode = Opcode::Push;
        }

        Ok(match instruction {
            ParsedInstruction::ZeroOp { opcode: _, span: _ } => {
                VerifiedInstruction::ZeroOp { opcode }
            }
            ParsedInstruction::OneOp {
                opcode: _,
                span,
                operand,
            } => {
                let verified = self.verify_operand(operand, &accepted_operands[0], 1, *span)?;

                VerifiedInstruction::OneOp {
                    opcode,
                    operand: verified,
                }
            }
            ParsedInstruction::TwoOp {
                opcode: _,
                span,
                operand1,
                operand2,
            } => {
                let verified1 = self.verify_operand(operand1, &accepted_operands[0], 1, *span)?;
                let verified2 = self.verify_operand(operand2, &accepted_operands[1], 2, *span)?;

                VerifiedInstruction::TwoOp {
                    opcode,
                    operand1: verified1,
                    operand2: verified2,
                }
            }
        })
    }

    // Verifies a single instruction operand
    fn verify_operand(
        &self,
        operand: &InstructionOperand,
        accepted: &[OperandType],
        num: usize,
        span: Span,
    ) -> Result<VerifiedOperand, ()> {
        match operand {
            InstructionOperand::Null => {
                if accepted.contains(&OperandType::Null) {
                    Ok(VerifiedOperand::Value(KOSValue::Null))
                } else {
                    self.error_invalid_operand(num, span, operand, accepted)?;

                    Err(())
                }
            }
            InstructionOperand::ArgMarker => {
                if accepted.contains(&OperandType::ArgMarker) {
                    Ok(VerifiedOperand::Value(KOSValue::ArgMarker))
                } else {
                    self.error_invalid_operand(num, span, operand, accepted)?;

                    Err(())
                }
            }
            InstructionOperand::Bool(b) => {
                if accepted.contains(&OperandType::Bool) {
                    Ok(VerifiedOperand::Value(KOSValue::Bool(*b)))
                } else if accepted.contains(&OperandType::BooleanValue) {
                    Ok(VerifiedOperand::Value(KOSValue::BoolValue(*b)))
                } else {
                    self.error_invalid_operand(num, span, operand, accepted)?;

                    Err(())
                }
            }
            InstructionOperand::String(s) => {
                if accepted.contains(&OperandType::String) {
                    Ok(VerifiedOperand::Value(KOSValue::String(s.clone())))
                } else if accepted.contains(&OperandType::StringValue) {
                    Ok(VerifiedOperand::Value(KOSValue::StringValue(s.clone())))
                } else {
                    self.error_invalid_operand(num, span, operand, accepted)?;

                    Err(())
                }
            }
            InstructionOperand::Float(f) => {
                if accepted.contains(&OperandType::Double) {
                    Ok(VerifiedOperand::Value(KOSValue::Double(*f)))
                } else if accepted.contains(&OperandType::ScalarDouble) {
                    Ok(VerifiedOperand::Value(KOSValue::ScalarDouble(*f)))
                } else {
                    self.error_invalid_operand(num, span, operand, accepted)?;

                    Err(())
                }
            }
            InstructionOperand::Label(l) => {
                if accepted.contains(&OperandType::Label) {
                    if let Some(label) = self.label_manager.get(l) {
                        Ok(VerifiedOperand::Label(label.value))
                    } else {
                        self.session
                            .struct_span_error(
                                span,
                                format!("instruction references unknown label `{}`", l),
                            )
                            .emit();

                        Err(())
                    }
                } else {
                    self.error_invalid_operand(num, span, operand, accepted)?;

                    Err(())
                }
            }
            InstructionOperand::Symbol(s) => {
                if let Some(symbol) = self.symbol_manager.get(s) {
                    if symbol.sym_type == SymbolType::Func {
                        if accepted.contains(&OperandType::Function) {
                            Ok(VerifiedOperand::Symbol(s.clone()))
                        } else {
                            self.error_invalid_operand(num, span, operand, accepted)?;

                            Err(())
                        }
                    } else if symbol.sym_type == SymbolType::Value {
                        // We can do a little error checking, but if it is external, we can't
                        // really do much

                        if symbol.binding.unwrap() != SymBind::Extern {
                            let is_ok = match &symbol.value {
                                SymbolValue::Value(value) => {
                                    let operand_type = match value {
                                        KOSValue::Byte(_) => OperandType::Byte,
                                        KOSValue::Int16(_) => OperandType::Int16,
                                        KOSValue::Int32(_) => OperandType::Int32,
                                        KOSValue::ScalarInt(_) => OperandType::ScalarInt,
                                        KOSValue::Double(_) => OperandType::Double,
                                        KOSValue::ScalarDouble(_) => OperandType::ScalarDouble,
                                        KOSValue::Bool(_) => OperandType::Bool,
                                        KOSValue::BoolValue(_) => OperandType::BooleanValue,
                                        KOSValue::String(_) => OperandType::String,
                                        KOSValue::StringValue(_) => OperandType::StringValue,
                                        KOSValue::Null => OperandType::Null,
                                        KOSValue::ArgMarker => OperandType::ArgMarker,
                                        KOSValue::Float(_) => unreachable!(),
                                    };

                                    accepted.contains(&operand_type)
                                }
                                SymbolValue::Function => accepted.contains(&OperandType::Function),
                                SymbolValue::Undefined => {
                                    self.session
                                        .struct_bug(
                                            "symbol being verified had Undefined value".to_string(),
                                        )
                                        .emit();

                                    return Err(());
                                }
                            };

                            if is_ok {
                                Ok(VerifiedOperand::Symbol(s.clone()))
                            } else {
                                self.error_invalid_operand(num, span, operand, accepted)?;

                                Err(())
                            }
                        } else {
                            // We just have to accept it
                            Ok(VerifiedOperand::Symbol(s.clone()))
                        }
                    } else {
                        self.session
                            .struct_bug("symbol being verified had the Default type".to_string())
                            .emit();

                        Err(())
                    }
                } else {
                    // This symbol doesn't exist
                    self.session
                        .struct_span_error(
                            span,
                            format!("instruction references symbol `{}`, that does not exist", s),
                        )
                        .emit();

                    Err(())
                }
            }
            InstructionOperand::Integer(i) => {
                if accepted.contains(&OperandType::Byte)
                    || accepted.contains(&OperandType::Int16)
                    || accepted.contains(&OperandType::Int32)
                    || accepted.contains(&OperandType::ScalarInt)
                {
                    let value = match self.maybe_squish_integer(*i, accepted) {
                        Ok(v) => v,
                        Err(_) => {
                            let largest = self.largest_accepted_integer(accepted)?;

                            self.session
                                .struct_error(format!(
                                    "instruction requires integer that can fit in a {}",
                                    largest
                                ))
                                .span_label(span, "integer value is too large to fit".to_string())
                                .emit();

                            return Err(());
                        }
                    };

                    Ok(VerifiedOperand::Value(value))
                } else {
                    self.error_invalid_operand(num, span, operand, accepted)?;

                    Err(())
                }
            }
        }
    }

    // This function is a binary size optimization function. If the instruction can allow for the
    // integer to be shortened down into 1 byte, and the value can fit, then it returns a
    // KOSValue::Byte(). It likewise tries the next size up, until it either finds that the
    // instruction doesn't support integers this large, or it finds the smallest size the integer
    // can fit.
    fn maybe_squish_integer(&self, value: i32, accepted: &[OperandType]) -> Result<KOSValue, ()> {
        let smallest_size = if <i8 as TryFrom<i32>>::try_from(value).is_ok() {
            OperandType::Byte
        } else if <i16 as TryFrom<i32>>::try_from(value).is_ok() {
            OperandType::Int16
        } else {
            OperandType::Int32
        };

        Ok(match smallest_size {
            OperandType::Byte => {
                if accepted.contains(&OperandType::Byte) {
                    KOSValue::Byte(value as i8)
                } else if accepted.contains(&OperandType::Int16) {
                    KOSValue::Int16(value as i16)
                } else if accepted.contains(&OperandType::Int32) {
                    KOSValue::Int32(value as i32)
                } else {
                    KOSValue::ScalarInt(value as i32)
                }
            }
            OperandType::Int16 => {
                if accepted.contains(&OperandType::Int16) {
                    KOSValue::Int16(value as i16)
                } else if accepted.contains(&OperandType::Int32) {
                    KOSValue::Int32(value as i32)
                } else if accepted.contains(&OperandType::ScalarInt) {
                    KOSValue::ScalarInt(value as i32)
                } else {
                    // If we have reached here, then the instruction needs a byte
                    return Err(());
                }
            }
            OperandType::Int32 => {
                if accepted.contains(&OperandType::Int32) {
                    KOSValue::Int32(value as i32)
                } else if accepted.contains(&OperandType::ScalarInt) {
                    KOSValue::ScalarInt(value as i32)
                } else {
                    // If we have reached here, then the instruction needs a byte or an int16
                    return Err(());
                }
            }
            _ => unreachable!(),
        })
    }

    // Returns a string that explains what the largest accepted integer type is for this
    // instruction
    fn largest_accepted_integer(&self, accepted: &[OperandType]) -> Result<&'static str, ()> {
        Ok(
            if accepted.contains(&OperandType::Int32) || accepted.contains(&OperandType::ScalarInt)
            {
                "32-bit integer"
            } else if accepted.contains(&OperandType::Int16) {
                "16-bit integer"
            } else if accepted.contains(&OperandType::Byte) {
                "8-bit integer"
            } else {
                self.session.struct_bug("tried to find smallest accepted integer, but the instruction does not take integers".to_string()).emit();

                return Err(());
            },
        )
    }

    fn error_invalid_operand(
        &self,
        num: usize,
        span: Span,
        provided: &InstructionOperand,
        accepted: &[OperandType],
    ) -> Result<(), ()> {
        let instr_snippet = self.session.span_to_snippet(&span);
        let instr_str = instr_snippet.as_slice();

        let accepted_types_s = self.accepted_types_to_string(accepted);
        let provided_str = provided.to_str();

        self.session
            .struct_error(format!(
                "instruction {} operand {} can be of the types: {}",
                instr_str, num, accepted_types_s
            ))
            .span_label(span, format!("found operand of type `{}`", provided_str))
            .emit();

        Err(())
    }

    fn accepted_types_to_string(&self, accepted: &[OperandType]) -> String {
        let mut s = String::new();

        if accepted.len() == 1 {
            s = accepted.first().unwrap().to_str().to_string();
        } else if accepted.len() == 2 {
            let first = accepted.first().unwrap().to_str();
            let second = accepted.last().unwrap().to_str();

            s = format!("{} or {}", first, second);
        } else {
            let second_to_last = accepted.iter().nth(accepted.len() - 2).unwrap().to_str();
            let last = accepted.last().unwrap().to_str();

            for op_type in accepted.iter().take(accepted.len() - 2) {
                s.push_str(&format!("{}, ", op_type.to_str()));
            }

            s.push_str(&format!("{} or {}", second_to_last, last));
        }

        s
    }

    fn lookup_accepted_operands(
        &self,
        opcode: Opcode,
    ) -> Result<&'static [&'static [OperandType]], ()> {
        Ok(match opcode {
            Opcode::Eof => &[&[]],
            Opcode::Eop => &[&[]],
            Opcode::Nop => &[&[]],
            Opcode::Sto => &[&[OperandType::String]],
            Opcode::Uns => &[&[]],
            Opcode::Gmb => &[&[OperandType::String]],
            Opcode::Smb => &[&[OperandType::String]],
            Opcode::Gidx => &[&[]],
            Opcode::Sidx => &[&[]],
            Opcode::Bfa => &[&[OperandType::String, OperandType::Int32, OperandType::Label]],
            Opcode::Jmp => &[&[OperandType::String, OperandType::Int32, OperandType::Label]],
            Opcode::Add => &[&[]],
            Opcode::Sub => &[&[]],
            Opcode::Mul => &[&[]],
            Opcode::Div => &[&[]],
            Opcode::Pow => &[&[]],
            Opcode::Cgt => &[&[]],
            Opcode::Clt => &[&[]],
            Opcode::Cge => &[&[]],
            Opcode::Cle => &[&[]],
            Opcode::Ceq => &[&[]],
            Opcode::Cne => &[&[]],
            Opcode::Neg => &[&[]],
            Opcode::Bool => &[&[]],
            Opcode::Not => &[&[]],
            Opcode::And => &[&[]],
            Opcode::Or => &[&[]],
            Opcode::Call => &[
                &[
                    OperandType::String,
                    OperandType::Null,
                    OperandType::Function,
                ],
                &[
                    OperandType::String,
                    OperandType::Int16,
                    OperandType::Int32,
                    OperandType::Null,
                ],
            ],
            Opcode::Ret => &[&[OperandType::Int16]],
            Opcode::Push => &[&[
                OperandType::Null,
                OperandType::Bool,
                OperandType::Byte,
                OperandType::Int16,
                OperandType::Int32,
                OperandType::String,
                OperandType::ArgMarker,
                OperandType::Double,
            ]],
            Opcode::Pop => &[&[]],
            Opcode::Dup => &[&[]],
            Opcode::Swap => &[&[]],
            Opcode::Eval => &[&[]],
            Opcode::Addt => &[&[OperandType::Bool], &[OperandType::Int32]],
            Opcode::Rmvt => &[&[]],
            Opcode::Wait => &[&[]],
            Opcode::Gmet => &[&[OperandType::String]],
            Opcode::Stol => &[&[OperandType::String]],
            Opcode::Stog => &[&[OperandType::String]],
            Opcode::Bscp => &[&[OperandType::Int16], &[OperandType::Int16]],
            Opcode::Escp => &[&[OperandType::Int16]],
            Opcode::Stoe => &[&[OperandType::String]],
            Opcode::Phdl => &[&[OperandType::Byte, OperandType::Int16, OperandType::Int32]],
            Opcode::Btr => &[&[OperandType::String, OperandType::Int32, OperandType::Label]],
            Opcode::Exst => &[&[]],
            Opcode::Argb => &[&[]],
            Opcode::Targ => &[&[]],
            Opcode::Tcan => &[&[]],

            Opcode::Prl => &[&[OperandType::String]],
            Opcode::Pdrl => &[
                &[OperandType::String, OperandType::Function],
                &[OperandType::Bool],
            ],
            Opcode::Lbrt => &[&[OperandType::String]],

            // Pseudo-instruction
            Opcode::Pushv => &[&[
                OperandType::Null,
                OperandType::BooleanValue,
                OperandType::ScalarInt,
                OperandType::StringValue,
                OperandType::ArgMarker,
                OperandType::ScalarDouble,
            ]],

            Opcode::Bogus => {
                self.session
                    .struct_bug("allowed bogus instruction to reach verifier".to_string())
                    .emit();

                return Err(());
            }
        })
    }
}
