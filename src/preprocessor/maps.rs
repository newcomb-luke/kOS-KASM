use std::collections::HashMap;

use super::past::{MLMacroArgs, MLMacroDef, MLMacroUndef, MacroInvok, SLMacroDef, SLMacroUndef};

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

    pub fn undefine(&mut self, sl_macro_undef: SLMacroUndef) {
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

    /// Returns a string explaining the combinations of different numbers of arguments
    /// that a given macro can receive
    pub fn get_accepted_num_args(&self, hash: u64) -> Option<String> {
        let overloaded_macros = self
            .map
            .values()
            .filter(|entry| entry.identifier.hash == hash);

        let mut arg_nums = Vec::new();

        for sl_macro in overloaded_macros {
            let num_args = sl_macro
                .args
                .as_ref()
                .map(|args| args.args.len() as u8)
                .unwrap_or(0);

            arg_nums.push(num_args);
        }

        if arg_nums.is_empty() {
            None
        } else {
            arg_nums.sort();

            Some(if arg_nums.len() == 1 {
                format!("{}", arg_nums.first().unwrap())
            } else if arg_nums.len() == 2 {
                format!(
                    "{} or {}",
                    arg_nums.first().unwrap(),
                    arg_nums.last().unwrap()
                )
            } else {
                let mut s = String::new();

                for num in arg_nums.iter().take(arg_nums.len() - 1) {
                    s.push_str(&format!("{}, ", num));
                }

                s.push_str(&format!("or {}", arg_nums.last().unwrap()));

                s
            })
        }
    }

    /// Returns true if a single-line macro with the identifier hash and number of arguments is
    /// defined in the map
    pub fn contains(&self, hash: u64, num_args: u8) -> bool {
        self.map.contains_key(&(hash, num_args))
    }

    /// Returns the first single-line macro defined with the given identifier hash or None if none
    /// exists with that hash
    pub fn find_by_hash(&self, hash: u64) -> Option<&SLMacroDef> {
        self.map
            .iter()
            .find(|((entry_hash, _), _)| *entry_hash == hash)
            .map(|((_, _), entry)| entry)
    }

    /// Returns true if a single-line macro with the identifier hash is defined in the map
    pub fn contains_hash(&self, hash: u64) -> bool {
        self.map.keys().find(|key| key.0 == hash).is_some()
    }
}

pub struct MLMacroMap {
    macros: Vec<(u64, MLMacroDef)>,
}

impl MLMacroMap {
    /// Creates a new empty MLMacroMap
    pub fn new() -> Self {
        Self { macros: Vec::new() }
    }

    /// Defines a new multi-line macro. This function returns true if this macro was redefined, and
    /// false otherwise.
    pub fn define(&mut self, ml_macro: MLMacroDef) -> bool {
        let hash = ml_macro.identifier.hash;

        let replace_index = self.find(hash, &ml_macro.args);

        if let Some(replace_index) = replace_index {
            self.macros.swap_remove(replace_index);
            self.macros.push((hash, ml_macro));

            true
        } else {
            self.macros.push((hash, ml_macro));

            false
        }
    }

    /// Undefines a multi-line macro if it exists
    pub fn undefine(&mut self, ml_macro_undef: MLMacroUndef) {
        let hash = ml_macro_undef.identifier.hash;

        let index = self.find(hash, &Some(ml_macro_undef.args));

        if let Some(index) = index {
            self.macros.swap_remove(index);
        }
    }

    /// Returns true if a multi-line macro with the identifier hash and argument range is defined
    /// in the map
    pub fn contains(&self, hash: u64, ml_args: &Option<MLMacroArgs>) -> bool {
        self.find(hash, ml_args).is_some()
    }

    /// Returns the first multi-line macro defined with the given identifier hash or None if none
    /// exists with that hash
    pub fn find_by_hash(&self, hash: u64) -> Option<&MLMacroDef> {
        self.macros
            .iter()
            .find(|entry| entry.0 == hash)
            .map(|entry| &entry.1)
    }

    /// Returns true if a multi-line macro with the identifier hash is defined in the map
    pub fn contains_hash(&self, hash: u64) -> bool {
        self.macros.iter().find(|entry| entry.0 == hash).is_some()
    }

    /// Gets a corresponding macro definition to a macro invokation, if it does match any in the
    /// map
    pub fn get(&self, invokation: &MacroInvok) -> Option<&MLMacroDef> {
        let hash = invokation.identifier.hash;

        let args = match &invokation.args {
            Some(args) => {
                let num = args.args.len() as u8;
                (num, num)
            }
            None => (0, 0),
        };

        for (macro_hash, ml_macro) in self.macros.iter() {
            let macro_range = Self::get_arg_range(&ml_macro.args);

            if hash == *macro_hash && Self::overlaps(args, macro_range) {
                return Some(ml_macro);
            }
        }

        return None;
    }

    // Returns a "range" with the None case being replaced with (0, 0), and the case where there is
    // no range and in fact only the required number (x) specified as (x, x)
    fn get_arg_range(ml_macro_args: &Option<MLMacroArgs>) -> (u8, u8) {
        match ml_macro_args {
            Some(args) => (
                args.required,
                args.maximum.map(|arg| arg.get()).unwrap_or(args.required),
            ),
            None => (0, 0),
        }
    }

    // Returns the index of the macro with overlapping macro arguments, or None if none is found
    fn find(&self, hash: u64, ml_args: &Option<MLMacroArgs>) -> Option<usize> {
        let range = Self::get_arg_range(&ml_args);
        let mut replace_index = None;

        for (index, (other_hash, other_macro)) in self.macros.iter().enumerate() {
            let other_range = Self::get_arg_range(&other_macro.args);

            if hash == *other_hash && Self::overlaps(range, other_range) {
                replace_index = Some(index);
                break;
            }
        }

        replace_index
    }

    fn overlaps(range1: (u8, u8), range2: (u8, u8)) -> bool {
        // https://stackoverflow.com/questions/3269434/whats-the-most-efficient-way-to-test-if-two-ranges-overlap
        range1.0 <= range2.1 && range2.0 <= range1.1
    }
}
