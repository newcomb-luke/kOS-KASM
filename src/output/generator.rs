use kerbalobjects::kofile::KOFile;

use crate::{
    parser::{LabelManager, SymbolManager},
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

    pub fn generate(mut self, functions: Vec<VerifiedFunction>) -> Result<KOFile, ()> {
        todo!();
    }
}
