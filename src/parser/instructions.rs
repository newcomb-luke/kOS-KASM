use crate::{Value};

pub struct Instruction {
    opcode: u8,
    operands: Vec<Value>,
}

pub struct Label {
    id: String
}

pub enum TextEntry {
    Instruction(Instruction),
    Label(Label)
}