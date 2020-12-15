use crate::{Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperandType {
    NULL,
    BOOL,
    BYTE,
    INT16,
    INT32,
    // FLOAT,
    // DOUBLE,
    // STRING,
    ARGMARKER,
    SCALARINT,
    SCALARDOUBLE,
    BOOLEANVALUE,
    STRINGVALUE
}

pub struct Instruction {}

impl Instruction {

    /// Returns true of the given string is a mnemonic of an instruction, false otherwise.
    pub fn is_instruction(mnemonic: &str) -> bool {
        let opcode = Instruction::opcode_from_mnemonic(mnemonic);
        // If the opcode is 0 that means it is bogus, or not an instruction
        opcode != 0
    }

    pub fn opcode_from_mnemonic(mnemonic: &str) -> u8 {
        match mnemonic {
            "eof"  => 0x31,
            "eop"  => 0x32,
            "nop"  => 0x33,
            "sto"  => 0x34,
            "uns"  => 0x35,
            "gmb"  => 0x36,
            "smb"  => 0x37,
            "gidx" => 0x38,
            "sidx" => 0x39,
            "bfa"  => 0x3a,
            "jmp"  => 0x3b,
            "add"  => 0x3c,
            "sub"  => 0x3d,
            "mul"  => 0x3e,
            "div"  => 0x3f,
            "pow"  => 0x40,
            "cgt"  => 0x41,
            "clt"  => 0x42,
            "cge"  => 0x43,
            "cle"  => 0x44,
            "ceq"  => 0x45,
            "cne"  => 0x46,
            "neg"  => 0x47,
            "bool" => 0x48,
            "not"  => 0x49,
            "and"  => 0x4a,
            "or"   => 0x4b,
            "call" => 0x4c,
            "ret"  => 0x4d,
            "push" => 0x4e,
            "pop"  => 0x4f,
            "dup"  => 0x50,
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
            "btr"  => 0x5e,
            "exst" => 0x5f,
            "argb" => 0x60,
            "targ" => 0x61,
            "tcan" => 0x62,

            "prl"  => 0xce,
            "pdrl" => 0xcd,
            "lbrt" => 0xf0,

            _ => 0x00
        }
    }

    // Returns a vector of vectors representing the different operand types that each instruction can take
    pub fn operands_from_opcode(opcode: u8) -> Vec<Vec<OperandType>> {
        match opcode {
            0x31 => vec![],
            0x32 => vec![],
            0x33 => vec![],
            0x34 => vec![vec![OperandType::STRINGVALUE]],
            0x35 => vec![],
            0x36 => vec![vec![OperandType::STRINGVALUE]],
            0x37 => vec![vec![OperandType::STRINGVALUE]],
            0x38 => vec![],
            0x39 => vec![],
            0x3a => vec![vec![OperandType::STRINGVALUE, OperandType::INT16, OperandType::INT32, OperandType::BYTE]],
            0x3b => vec![vec![OperandType::STRINGVALUE, OperandType::INT16, OperandType::INT32, OperandType::BYTE]],
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
            0x4c => vec![vec![OperandType::STRINGVALUE], vec![OperandType::STRINGVALUE, OperandType::INT16, OperandType::INT32]],
            0x4d => vec![vec![OperandType::INT16]],
            0x4e => vec![vec![OperandType::NULL, OperandType::BOOLEANVALUE, OperandType::BYTE, OperandType::INT16, OperandType::SCALARINT, OperandType::SCALARDOUBLE, OperandType::STRINGVALUE, OperandType::ARGMARKER]],
            0x4f => vec![],
            0x50 => vec![],
            0x51 => vec![],
            0x52 => vec![],
            0x53 => vec![vec![OperandType::BOOL], vec![OperandType::INT32]],
            0x54 => vec![],
            0x55 => vec![],
            0x56 => vec![],
            0x57 => vec![vec![OperandType::STRINGVALUE]],
            0x58 => vec![vec![OperandType::STRINGVALUE]],
            0x59 => vec![vec![OperandType::STRINGVALUE]],
            0x5a => vec![vec![OperandType::INT16], vec![OperandType::INT16]],
            0x5b => vec![vec![OperandType::INT16]],
            0x5c => vec![vec![OperandType::STRINGVALUE]],
            0x5d => vec![vec![OperandType::BYTE, OperandType::INT16, OperandType::INT32]],
            0x5e => vec![vec![OperandType::STRINGVALUE, OperandType::INT16, OperandType::INT32, OperandType::BYTE]],
            0x5f => vec![],
            0x60 => vec![],
            0x61 => vec![],
            0x62 => vec![],

            0xce => vec![vec![OperandType::STRINGVALUE]],
            0xcd => vec![vec![OperandType::STRINGVALUE], vec![OperandType::BOOL]],
            0xf0 => vec![vec![OperandType::STRINGVALUE]],
            _ => vec![],
        }
    }

}