use crate::{Value};

pub struct Instruction {
    opcode: u8,
    operands: Vec<Value>,
}

impl Instruction {

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

            _ => 0x00
        }
    }

}

pub struct Label {
    id: String
}

pub enum TextEntry {
    Instruction(Instruction),
    Label(Label)
}