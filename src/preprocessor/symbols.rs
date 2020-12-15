use std::{collections::HashMap, error::Error};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolType {
    LABEL,
    FUNC,
    UNDEFFUNC,
    DATA,
    UNDEF
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolInfo {
    GLOBAL,
    LOCAL,
    EXTERN
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolValue {
    NONE,
    BOOL(bool),
    INT(i32),
    DOUBLE(f64),
    STRING(String)
}

#[derive(Debug, Clone)]
pub struct Symbol {
    id: String,
    st: SymbolType,
    si: SymbolInfo,
    sv: SymbolValue,
}

pub struct SymbolManager {
    symbols: HashMap<String, Symbol>,
}

impl Symbol {
    pub fn new(identifier: &str, st: SymbolType, si: SymbolInfo, sv: SymbolValue) -> Symbol {
        Symbol {
            id: identifier.to_owned(),
            st,
            si,
            sv
        }
    }

    pub fn sym_type(&self) -> SymbolType {
        self.st
    }

    pub fn sym_info(&self) -> SymbolInfo {
        self.si
    }

    pub fn sym_value(&self) -> &SymbolValue {
        &self.sv
    }

    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn as_str(&self) -> String {
        format!("{}: {:?}, {:?}, {:?}", self.id, self.st, self.si, self.sv)
    }
}

impl SymbolManager {
    pub fn new() -> SymbolManager {
        SymbolManager { symbols: HashMap::new() }
    }

    pub fn def(&mut self, identifier: &str, symbol: Symbol) {
        // This already does what it needs to do. If it exists, the value is updated, if not, the value is created.
        self.symbols
        .insert(String::from(identifier), symbol);
    }

    pub fn ifdef(&self, identifier: &str) -> bool {
        self.symbols.contains_key(identifier)
    }

    pub fn get(&self, identifier: &str) -> Result<&Symbol, Box<dyn Error>> {
        if self.ifdef(identifier) {
            Ok(self.symbols.get(identifier).unwrap())
        } else {
            Err(format!("Constant {} referenced before definition", identifier).into())
        }
    }

    pub fn as_vec(&self) -> Vec<&Symbol> {
        let mut all_values = Vec::new();

        for sym in self.symbols.values() {
            all_values.push(sym);
        }

        all_values
    }
}