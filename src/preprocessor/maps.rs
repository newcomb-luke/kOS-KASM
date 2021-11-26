use std::collections::HashMap;

use super::past::{MacroInvok, SLMacroDef, SLMacroUndef};

pub struct SLMacroMap {
    map: HashMap<(u64, u8), SLMacroDef>,
}

impl SLMacroMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn define(&mut self, sl_macro: SLMacroDef) {
        let hash = sl_macro.identifier.hash;
        let args = match &sl_macro.args {
            Some(args) => args.args.len() as u8,
            None => 0,
        };

        self.map.insert((hash, args), sl_macro);
    }

    pub fn undefine(&mut self, sl_macro_undef: &SLMacroUndef) {
        let hash = sl_macro_undef.identifier.hash;
        let args = sl_macro_undef.args.num;

        self.map.remove(&(hash, args));
    }

    pub fn get(&self, invokation: &MacroInvok) -> Option<&SLMacroDef> {
        let hash = invokation.identifier.hash;
        let args = match &invokation.args {
            Some(args) => args.args.len() as u8,
            None => 0,
        };

        self.map.get(&(hash, args))
    }

    pub fn contains(&self, hash: u64, num_args: u8) -> bool {
        self.map.contains_key(&(hash, num_args))
    }
}
